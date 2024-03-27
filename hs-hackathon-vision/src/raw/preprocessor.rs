use crate::raw::led_detector::LedDetectionConfig;
use eyre::eyre;
use image::{DynamicImage, GenericImageView, ImageBuffer, Luma, Rgba};

pub struct BrightArea {
    pub thresholded: ImageBuffer<Luma<u8>, Vec<u8>>,
    pub resized: DynamicImage,
}

pub fn extract_bright_areas(
    image: &DynamicImage,
    config: &LedDetectionConfig,
) -> eyre::Result<BrightArea> {
    // Resize to make blur fast enough
    let resized = image.resize(config.width, config.height, config.filter);

    // Difference of gaussians highlights bright areas in the image.
    let bright_areas = difference_of_gaussians(&resized, config.radius_1, config.radius_2)?;
    // Keep only the "very bright" areas (255) and discard everything else (0)
    let thresholded = threshold(&bright_areas, config.threshold_value);
    Ok(BrightArea {
        thresholded,
        resized,
    })
}

fn subtract(img1: &DynamicImage, img2: &DynamicImage) -> eyre::Result<DynamicImage> {
    if img1.dimensions() != img2.dimensions() {
        return Err(eyre!("Images must have the same dimensions"));
    }

    // TODO: optimise this code; should be able to do this on the underlying array
    let mut result = ImageBuffer::new(img1.width(), img1.height());
    for (x, y, pixel) in img1.pixels() {
        let img2_pixel = img2.get_pixel(x, y);
        let subtracted_pixel = Rgba([
            pixel[0].saturating_sub(img2_pixel[0]),
            pixel[1].saturating_sub(img2_pixel[1]),
            pixel[2].saturating_sub(img2_pixel[2]),
            255, // Assuming alpha channel remains fully opaque
        ]);

        result.put_pixel(x, y, subtracted_pixel);
    }
    Ok(DynamicImage::ImageRgba8(result))
}

fn difference_of_gaussians(
    img: &DynamicImage,
    radius1: f32,
    radius2: f32,
) -> eyre::Result<DynamicImage> {
    subtract(&img.blur(radius1), &img.blur(radius2))
}

fn threshold(img: &DynamicImage, threshold: u8) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let gray_img = img.to_luma8();
    let (width, height) = gray_img.dimensions();

    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = gray_img.get_pixel(x, y);
        if pixel[0] > threshold {
            Luma([0u8]) // Black
        } else {
            Luma([255u8]) // White
        }
    })
}
