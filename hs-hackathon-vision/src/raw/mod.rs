#![allow(unused)]

pub mod bounding_box;
pub mod colors;
pub mod distance;
pub mod led_detector;
pub mod preprocessor;
pub mod utils;

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];

const WHITE: [u8; 4] = [255, 255, 255, 255];

// Values from https://convertingcolors.com/
// https://en.wikipedia.org/wiki/CIELAB_color_space
const LAB_RED: (f32, f32, f32) = (53.24, 80.09, 67.20);
const LAB_BLUE: (f32, f32, f32) = (32.30, 79.19, -107.86);
const LAB_GREEN: (f32, f32, f32) = (87.73, -86.18, 83.18);

const LAB_WHITE: (f32, f32, f32) = (100., 0.51, -0.37);

// Using HSV for white LEDs to improve the accuracy
const WHITE_SATURATION_THRESHOLD: f32 = 0.35;
const WHITE_VALUE_THRESHOLD: f32 = 0.09;
const WHITE_BRIGHTNESS_THRESHOLD: u8 = 250;

#[cfg(test)]
mod tests {
    use crate::raw::bounding_box::BoundingBox;
    use crate::raw::colors::Color;
    use crate::raw::distance::centroid_distance;
    use crate::raw::led_detector::{get_leds, LedDetectionConfig};
    use crate::raw::utils::draw_bounding_boxes;
    use crate::raw::{BLUE, GREEN, RED};
    use image::imageops::FilterType;
    use image::Rgba;
    use palette::named::WHITE;
    use std::env;

    #[test]
    fn test_distance_red_green_blue() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/test_distance.png");
        let mut img = image::open(path).unwrap();

        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (10, 10),
            max_size: (50, 50),
        };

        let leds = get_leds(&mut img, &config).unwrap();

        let red_led = leds
            .iter()
            .find(|led| led.color == Color::Red)
            .expect("Can't find red led");
        let green_led = leds
            .iter()
            .find(|led| led.color == Color::Green)
            .expect("Can't find green led");
        let blue_led = leds
            .iter()
            .find(|led| led.color == Color::Blue)
            .expect("Can't find blue led");

        draw_bounding_boxes(&mut img, vec![red_led.bbox], Rgba(RED));
        draw_bounding_boxes(&mut img, vec![green_led.bbox], Rgba(GREEN));
        draw_bounding_boxes(&mut img, vec![blue_led.bbox], Rgba(BLUE));
        img.save("overlay_red_green_blue.png").unwrap();
        let distance_r_g = centroid_distance(red_led.bbox, green_led.bbox);
        let distance_r_b = centroid_distance(red_led.bbox, blue_led.bbox);
        let distance_b_g = centroid_distance(blue_led.bbox, green_led.bbox);
        assert_eq!(distance_r_g, 267);
        assert_eq!(distance_r_b, 143);
        assert_eq!(distance_b_g, 409);
    }

    #[test]
    fn detect_green() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/car_green.png");
        let mut img = image::open(path).unwrap();

        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 20,
            min_size: (10, 10),
            max_size: (20, 20),
        };

        let leds = get_leds(&mut img, &config).unwrap();
        assert_eq!(leds.len(), 1);
        assert_eq!(leds[0].color, Color::Green);
        assert_eq!(leds[0].bbox.x_min(), 586);
        assert_eq!(leds[0].bbox.y_min(), 730);
        assert_eq!(leds[0].bbox.x_max(), 617);
        assert_eq!(leds[0].bbox.y_max(), 774);
        draw_bounding_boxes(&mut img, vec![leds[0].bbox], Rgba(GREEN));
        img.save("car_green_bbox.png").unwrap();
    }

    #[test]
    fn detect_red() {
        let path = env::current_dir().unwrap().join("../resources/car_red.png");
        let mut img = image::open(path).unwrap();

        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 20,
            min_size: (7, 7),
            max_size: (20, 20),
        };

        let leds = get_leds(&mut img, &config).unwrap();
        draw_bounding_boxes(&mut img, vec![leds[0].bbox], Rgba(RED));
        img.save("car_red_bbox.png").unwrap();
        assert_eq!(leds.len(), 1);
        assert_eq!(leds[0].color, Color::Red);
        assert_eq!(leds[0].bbox.x_min(), 625);
        assert_eq!(leds[0].bbox.y_min(), 878);
        assert_eq!(leds[0].bbox.x_max(), 651);
        assert_eq!(leds[0].bbox.y_max(), 904);
    }

    #[test]
    fn detect_blue() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/car_blue.png");
        let mut img = image::open(path).unwrap();

        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 20,
            min_size: (7, 7),
            max_size: (20, 20),
        };

        let leds = get_leds(&mut img, &config).unwrap();
        draw_bounding_boxes(&mut img, vec![leds[0].bbox], Rgba(BLUE));
        img.save("car_blue_bbox.png").unwrap();
        assert_eq!(leds.len(), 1);
        assert_eq!(leds[0].color, Color::Blue);
        assert_eq!(leds[0].bbox.x_min(), 633);
        assert_eq!(leds[0].bbox.y_min(), 856);
        assert_eq!(leds[0].bbox.x_max(), 658);
        assert_eq!(leds[0].bbox.y_max(), 887);
    }
}
