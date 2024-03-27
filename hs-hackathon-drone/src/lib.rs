use eyre::Context;
use image::{codecs::jpeg::JpegDecoder, DynamicImage};

/// A connection to the camera of the drone and abstraction to access the drones camera
pub struct Camera(reqwest::Client);

/// A videoframe recieved from the drones camera
#[derive(Clone)]
pub struct Frame(pub DynamicImage);

impl Camera {
    pub async fn connect() -> color_eyre::Result<Self> {
        Ok(Self(reqwest::Client::new()))
    }

    pub async fn snapshot(&self) -> color_eyre::Result<Frame> {
        let res = self
            .0
            .get("http://127.0.0.1:3000/camera?clean=true")
            .send()
            .await
            .wrap_err("request image")?;
        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.wrap_err("fetch image error text")?;
            return Err(eyre::eyre!(body)).wrap_err(format!("image grab gave {status:?}"));
        };
        let bytes = res.bytes().await.wrap_err("read bytes")?;
        let decoder = JpegDecoder::new(&*bytes).wrap_err("launch decoder")?;
        let img = DynamicImage::from_decoder(decoder).wrap_err("decode frame")?;
        Ok(Frame(img))
    }
}
