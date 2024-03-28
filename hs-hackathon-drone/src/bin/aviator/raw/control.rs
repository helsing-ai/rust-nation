use crate::raw::{SND_ADDR, SND_PORT};
use clap::Subcommand;
use color_eyre::{eyre::WrapErr, Result};
use std::{net::SocketAddr, time::Duration};
use strum::Display;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;

#[allow(unused_imports)]
use tracing::{debug, info, trace, warn};

/// Information on what each command does can be found in the
/// [Tello docs](https://dl-cdn.ryzerobotics.com/downloads/Tello/Tello%20SDK%202.0%20User%20Guide.pdf).
#[derive(Subcommand, Display, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    #[strum(to_string = "command")]
    SDKInit,
    #[strum(to_string = "takeoff")]
    Takeoff,
    #[strum(to_string = "land")]
    Land,
    #[strum(to_string = "streamon")]
    EnableStream,
    #[strum(to_string = "up 20")]
    GoHigher,
    #[strum(to_string = "down 20")]
    GoLower,
    #[strum(to_string = "forward 20")]
    TrimFwd,
    #[strum(to_string = "back 20")]
    TrimBwd,
    #[strum(to_string = "left 20")]
    TrimLeft,
    #[strum(to_string = "right 20")]
    TrimRight,
    #[strum(to_string = "cw 10")]
    RotateCw,
    #[strum(to_string = "ccw 10")]
    RotateCcw,
    #[strum(to_string = "wifi {ssid} {pass}")]
    SetSsidPass {
        #[clap(long)]
        ssid: String,
        #[clap(long)]
        pass: String,
    },
    #[strum(to_string = "stop")]
    Stop,
    #[strum(to_string = "wifi?")]
    QueryWifi,
    #[strum(to_string = "battery?")]
    QueryBattery,
    #[strum(to_string = "time?")]
    QueryFlightTime,
}

/// Sending commands from [src] to the drone. Blocks until [src] is closed.
pub async fn send_commands(
    mut src: tokio::sync::mpsc::Receiver<(Command, tokio::sync::oneshot::Sender<String>)>,
) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8889").await?;
    let remote_addr = SocketAddr::from((SND_ADDR, SND_PORT));

    debug!("connecting to: {}", &remote_addr);

    let mut ack: Option<oneshot::Sender<String>> = None;
    let mut is_slow = false;
    let mut buf = [0u8; 2000];
    loop {
        let wait = if is_slow { 30 } else { 7 };
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(wait)), if ack.is_some() => {
                warn!("not getting ack from drone; is it on?");
                // fail the pending operation
                ack = None;
            }
            res = socket.recv(&mut buf) => {
                trace!("got ack");

                let size = res.wrap_err("recv_from")?;
                if size == 0 {
                    warn!("got empty UDP packet");
                    continue;
                }

                let msg = match std::str::from_utf8(&buf[..size]) {
                    Ok(s) => s,
                    Err(_) => {
                        warn!("got invalid utf-8 from drone: {:?}", &buf[..size]);
                        continue;
                    }
                };

                debug!("rcv: {}", msg);

                if let Some(ack) = ack.take() {
                    let _ = ack.send(String::from(msg));
                } else {
                    info!("unexpected ack from drone: {msg}");
                }
            }
            cmd = src.recv() => {
                if let Some((cmd, sink)) = cmd {
                    ack = Some(sink);
                    is_slow =  cmd == Command::SDKInit || cmd == Command::Takeoff;
                    debug!("snd: {}", cmd);
                    socket.send_to(cmd.to_string().as_bytes(), remote_addr).await.wrap_err("send cmd")?;
                } else {
                    info!("no more commands -- exiting");
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_serde() {
        assert_eq!("command", Command::SDKInit.to_string());
        assert_eq!("takeoff", Command::Takeoff.to_string());
        assert_eq!("land", Command::Land.to_string());
    }
}
