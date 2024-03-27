use crate::raw::bounding_box::BoundingBox;

pub fn centroid(bbox: BoundingBox) -> (u32, u32) {
    (
        (bbox.x_min() + bbox.x_max()) / 2,
        (bbox.y_min() + bbox.y_max()) / 2,
    )
}

/// Euclidean distance between bounding boxes centroids
pub fn centroid_distance(bbox_1: BoundingBox, bbox_2: BoundingBox) -> u32 {
    let bbox_1_center = centroid(bbox_1);
    let bbox_2_center = centroid(bbox_2);
    let dx = (bbox_2_center.0 as i32).saturating_sub(bbox_1_center.0 as i32);
    let dy = (bbox_2_center.1 as i32).saturating_sub(bbox_1_center.1 as i32);
    let distance_squared = dx.pow(2) + dy.pow(2);
    let distance = (distance_squared as f32).sqrt().round();

    distance as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let bbox_1 = BoundingBox::new(0, 0, 100, 100).unwrap();
        let bbox_2 = BoundingBox::new(100, 100, 200, 200).unwrap();
        assert_eq!(centroid(bbox_1), (50, 50));
        assert_eq!(centroid(bbox_2), (150, 150));
        assert_eq!(centroid_distance(bbox_1, bbox_2), 141);
    }
}
