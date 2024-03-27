pub(crate) mod raw;

use hs_hackathon_drone::Frame;
use image::DynamicImage;

use crate::raw::distance::centroid_distance;
use crate::raw::led_detector::get_leds;
pub use raw::bounding_box::BoundingBox;
pub use raw::colors::Color;
pub use raw::led_detector::{Led, LedDetectionConfig};

/// Detect all LEDs that are visible in a given frame
pub fn detect(frame: &Frame, configuration: &LedDetectionConfig) -> eyre::Result<Vec<Led>> {
    let dyn_image: DynamicImage = frame.0.clone().into();
    get_leds(&dyn_image, configuration)
}

/// Get distance between two LEDs
pub fn distance(led_1: &Led, led_2: &Led) -> u32 {
    centroid_distance(led_1.bbox, led_2.bbox)
}
