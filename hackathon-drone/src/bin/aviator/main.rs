use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use eyre::Context;
use image::{DynamicImage, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Cursor,
    net::SocketAddr,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot, watch, Mutex},
};
use tracing_core::LevelFilter;
use tracing_subscriber::EnvFilter;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

mod raw;
use raw::control::Command;

pub const FONT_DATA: &[u8] = include_bytes!("../../../../resources/DejaVuSans.ttf");
static FONT: OnceLock<Font<'static>> = OnceLock::new();

struct AppState {
    camera: watch::Receiver<image::RgbImage>,
    drone: Mutex<Drone>,
}

struct Drone {
    battery: i32,
    altitude: i32,
    moved_x: i8,
    moved_y: i8,
    task: mpsc::Sender<(Command, tokio::sync::oneshot::Sender<String>)>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .expect("internal error: failed to setup tracing");
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let (frame_tx, frame_rx) = watch::channel(RgbImage::new(960, 720));
    let (command_tx, command_rx) = mpsc::channel(1);
    let (syn, ack) = oneshot::channel();
    command_tx.try_send((Command::SDKInit, syn)).unwrap();
    let drone = Drone {
        battery: 0,
        altitude: 0,
        moved_x: 0,
        moved_y: 0,
        task: command_tx,
    };
    let shared_state = Arc::new(AppState {
        drone: Mutex::new(drone),
        camera: frame_rx,
    });

    // spawn video capturer
    let vidcap = tokio::spawn(async move {
        let recv_socket = UdpSocket::bind(SocketAddr::from((raw::VID_ADDR, raw::VID_PORT)))
            .await
            .wrap_err("bind to video receive socket")?;
        raw::h264::watch_latest_frame(frame_tx, recv_socket)
            .await
            .wrap_err("watch for h264 frames")?;
        Ok::<_, color_eyre::Report>(())
    });
    // spawn command dispatcher
    let dispatcher = tokio::spawn(async move {
        raw::control::tokio::send_commands(command_rx)
            .await
            .wrap_err("open command loop")?;
        Ok::<_, color_eyre::Report>(())
    });
    // spawn state tracker
    let for_spawn = Arc::clone(&shared_state);
    let tracker = tokio::spawn(async move {
        let shared_state = for_spawn;
        let socket = UdpSocket::bind(SocketAddr::from((raw::RCV_ADDR, raw::RCV_PORT)))
            .await
            .wrap_err("bind tracker")?;
        let mut buffer = [0u8; 2000];
        loop {
            let size = socket.recv(&mut buffer).await.wrap_err("recv")?;
            let received = String::from_utf8(buffer[0..size].to_vec())?;
            if let Ok(raw::sensors::State { h, bat, .. }) = received.parse::<raw::sensors::State>()
            {
                let mut drone = shared_state.drone.lock().await;
                drone.battery = bat;
                drone.altitude = h;
            } else {
                warn!("Invalid drone state: {received}");
            }
        }
        #[allow(unreachable_code)]
        Ok::<_, color_eyre::Report>(())
    });

    // wait for sdk init to be acked
    ack.await.wrap_err("ack sdk-init")?;

    // start the video stream
    let (syn, ack) = oneshot::channel();
    {
        shared_state
            .drone
            .lock()
            .await
            .task
            .try_send((Command::EnableStream, syn))
            .unwrap();
    }
    ack.await.wrap_err("ack enable-stream")?;

    // every 5 seconds, heartbeat
    let for_spawn = Arc::clone(&shared_state);
    tokio::spawn(async move {
        let shared_state = for_spawn;
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let (syn, ack) = oneshot::channel();
            let drone = shared_state.drone.lock().await;
            if let Err(e) = drone.task.try_send((Command::Stop, syn)) {
                match e {
                    mpsc::error::TrySendError::Closed(_) => {
                        debug!("exiting since command channel is closed");
                        // time to exit
                        break Ok::<_, color_eyre::Report>(());
                    }
                    mpsc::error::TrySendError::Full(_) => {
                        // fine -- means there are commands flowing
                        continue;
                    }
                }
            }
            drop(drone);
            ack.await.wrap_err("heartbeat")?;
        }
    });

    let server = tokio::spawn(async move {
        let app = Router::new()
            .route("/", get(root))
            .route("/camera", get(camera))
            .route("/nudge", post(nudge))
            .with_state(shared_state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        axum::serve(listener, app).await?;
        #[allow(unreachable_code)]
        Ok::<_, color_eyre::Report>(())
    });

    match tokio::try_join!(vidcap, dispatcher, tracker, server) {
        Ok((vidcap, dispatcher, tracker, server)) => {
            if let Err(e) = vidcap {
                error!("video capture task failed: {e:?}");
            }
            if let Err(e) = dispatcher {
                error!("dispatcher failed: {e:?}");
            }
            if let Err(e) = tracker {
                error!("drone state tracker failed: {e:?}");
            }
            if let Err(e) = server {
                error!("http server failed: {e:?}");
            }
        }
        Err(e) => {
            error!("tokio task panicked: {e:?}");
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Direction {
    Takeoff,
    Land,
    Left,
    Right,
    Forward,
    Backward,
    Up,
    Down,
}

struct Oof(StatusCode, String);

impl IntoResponse for Oof {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

impl From<color_eyre::Report> for Oof {
    fn from(value: color_eyre::Report) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, format!("{value:?}"))
    }
}

async fn root(State(_): State<Arc<AppState>>) {}

async fn camera(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, Oof> {
    let bat = {
        let drone = state.drone.lock().await;
        drone.battery
    };

    let image = state.camera.borrow();
    let mut dyn_image: DynamicImage = RgbImage::clone(&image).into();
    drop(image);

    if !params.contains_key("clean") {
        // draw battery %
        // TODO: also draw altitude warning?
        let font = FONT.get_or_init(|| Font::try_from_bytes(FONT_DATA).expect("load font"));
        draw_text_mut(
            &mut dyn_image,
            [0, 255, 0, 128].into(),
            5,
            5,
            Scale::uniform(25.0),
            &font,
            format!("Battery: {:02}%", bat).as_str(),
        );
    }

    let mut bytes = Cursor::new(Vec::new());
    dyn_image
        .write_to(&mut bytes, image::ImageFormat::Jpeg)
        .wrap_err("write image")?;

    Ok(([(header::CONTENT_TYPE, "image/jpeg")], bytes.into_inner()))
}

async fn nudge(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Direction>,
) -> Result<impl IntoResponse, Oof> {
    loop {
        let mut current = state.drone.lock().await;
        let invoke = match payload {
            Direction::Left => {
                if current.moved_x < -2 {
                    return Ok(());
                }
                current.moved_x -= 1;
                raw::control::Command::TrimLeft
            }
            Direction::Right => {
                if current.moved_x > 2 {
                    return Ok(());
                }
                current.moved_x += 1;
                raw::control::Command::TrimRight
            }
            Direction::Backward => {
                if current.moved_y < -2 {
                    return Ok(());
                }
                current.moved_y -= 1;
                raw::control::Command::TrimBwd
            }
            Direction::Forward => {
                if current.moved_y > 2 {
                    return Ok(());
                }
                current.moved_y += 1;
                raw::control::Command::TrimFwd
            }
            Direction::Up => {
                if current.altitude > 160 {
                    return Ok(());
                }
                raw::control::Command::GoHigher
            }
            Direction::Down => raw::control::Command::GoLower,
            Direction::Takeoff => raw::control::Command::Takeoff,
            Direction::Land => raw::control::Command::Land,
        };

        let (syn, ack) = tokio::sync::oneshot::channel();
        if let Err(e) = current.task.try_send((invoke, syn)) {
            match e {
                mpsc::error::TrySendError::Full(_) => {
                    // try again -- concurrent command
                    drop(current);
                    continue;
                }
                mpsc::error::TrySendError::Closed(_) => {
                    return Err(eyre::eyre!("can't send more commands -- loop terminated").into());
                }
            }
        }
        ack.await.wrap_err("ack cmd")?;
    }
}
