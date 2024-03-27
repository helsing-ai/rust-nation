use crate::raw::bounding_box::BoundingBox;
use image::{DynamicImage, GenericImage, Rgba};

pub fn draw_bounding_box(img: &mut DynamicImage, bbox: BoundingBox, border_color: Rgba<u8>) {
    if bbox.x_max() > bbox.x_min() {
        for x in bbox.x_min()..=bbox.x_max() {
            img.put_pixel(x, bbox.y_min(), border_color); // Top border
            img.put_pixel(x, bbox.y_max(), border_color); // Bottom border
        }
    }

    // Draw left and right border
    if bbox.y_max() > bbox.y_min() {
        for y in bbox.y_min()..=bbox.y_max() {
            img.put_pixel(bbox.x_min(), y, border_color); // Left border
            img.put_pixel(bbox.x_max(), y, border_color); // Right border
        }
    }
}
pub fn draw_bounding_boxes(
    img: &mut DynamicImage,
    bboxes: Vec<BoundingBox>,
    border_color: Rgba<u8>,
) {
    for bbox in bboxes {
        draw_bounding_box(img, bbox, border_color)
    }
}

pub fn bbox_resize(
    bbox: &BoundingBox,
    original_size: &(u32, u32),
    resized_size: &(u32, u32),
) -> eyre::Result<BoundingBox> {
    let scale_x = original_size.0 as f32 / resized_size.0 as f32;
    let scale_y = original_size.1 as f32 / resized_size.1 as f32;

    BoundingBox::new(
        (bbox.x_min() as f32 * scale_x).round() as u32,
        (bbox.y_min() as f32 * scale_y).round() as u32,
        (bbox.x_max() as f32 * scale_x).round() as u32,
        (bbox.y_max() as f32 * scale_y).round() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use std::env;

    #[test]
    fn test_drawing_bounding_boxes() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/test_multiple_bulbs.png");

        let path_with_bbox = env::current_dir()
            .unwrap()
            .join("../resources/test_multiple_bulbs_with_bbox.png");

        let mut img = image::open(path).unwrap();
        let img_with_bbox = image::open(path_with_bbox).unwrap();
        let bbox = BoundingBox::new(100, 100, 120, 120).unwrap();
        draw_bounding_boxes(&mut img, vec![bbox], Rgba([255, 255, 255, 255]));

        assert_eq!(img, img_with_bbox)
    }

    #[test]
    fn test_adapt_to_original_size() {
        let original_size = &(500, 800);
        let resized_size = &(250, 400);
        let bbox = BoundingBox::new(100, 100, 120, 120).unwrap();
        let adapted_bbox = bbox_resize(&bbox, original_size, resized_size).unwrap();
        let ratio_x = original_size.0 as f32 / resized_size.0 as f32;
        let ratio_y = original_size.1 as f32 / resized_size.1 as f32;
        assert_eq!(
            adapted_bbox.x_max() - adapted_bbox.x_min(),
            (bbox.x_max() - bbox.x_min()) * ratio_x.round() as u32
        );
        assert_eq!(
            adapted_bbox.y_max() - adapted_bbox.y_min(),
            (bbox.y_max() - bbox.y_min()) * ratio_y.round() as u32
        );
    }
}
