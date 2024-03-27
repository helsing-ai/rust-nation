use super::TeamColors;
use hs_hackathon_drone::Camera;
use hs_hackathon_vision::{BoundingBox, LedDetectionConfig};

/// A utility inference function
#[doc(hidden)]
pub(crate) async fn infer(
    colors: &TeamColors,
    camera: &Camera,
) -> eyre::Result<(BoundingBox, BoundingBox)> {
    loop {
        let frame = camera.snapshot().await?;

        let leds = crate::vision::detect(&frame, &LedDetectionConfig::default())?;

        let Some(car) = leds.iter().find(|led| led.color == colors.car) else {
            continue;
        };

        let Some(target) = leds.iter().find(|led| led.color == colors.target) else {
            continue;
        };

        return Ok((car.bbox, target.bbox));
    }
}
