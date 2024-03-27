use color_eyre::Report;
use hackathon_drone::Camera;
use image::DynamicImage;

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let camera = Camera::connect().await?;
    let image = camera.snapshot().await?;

    let dyn_image: DynamicImage = image.0.into();

    println!("Saved capture as `out.jpg`");
    dyn_image.save("out.jpg")?;

    Ok(())
}
