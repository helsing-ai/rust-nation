use crate::raw::{
    bounding_box::BoundingBox, BLUE, GREEN, LAB_BLUE, LAB_GREEN, LAB_RED, LAB_WHITE, RED,
    WHITE_BRIGHTNESS_THRESHOLD, WHITE_SATURATION_THRESHOLD, WHITE_VALUE_THRESHOLD,
};
use eyre::ContextCompat;
use image::{DynamicImage, GenericImageView, Rgb, Rgba};
use palette::{
    color_difference::EuclideanDistance,
    encoding::{Linear, Srgb as GammaSrgb},
    white_point::D65,
    FromColor, Hsv, Lab, LinSrgb, Srgb, Srgba,
};
use std::collections::HashMap;

#[derive(PartialEq, Clone, Copy, Eq, Hash, Debug)]
pub enum Color {
    Red,
    Blue,
    Green,
    White,
    Unknown,
}

impl Color {
    pub fn lab_rgb(&self) -> Lab {
        match *self {
            Color::Red => Lab::from_components(LAB_RED),
            Color::Blue => Lab::from_components(LAB_BLUE),
            Color::Green => Lab::from_components(LAB_GREEN),
            Color::White => Lab::from_components(LAB_WHITE),
            Color::Unknown => {
                panic!("Unknow colour, current detected colours are White/Red/Blue/Green")
            }
        }
    }
}

pub const fn colors() -> &'static [Color] {
    &[Color::Red, Color::Blue, Color::Green]
}

fn is_white(pixel: Rgba<u8>) -> bool {
    let max_pixel = u8::max(pixel[0], u8::max(pixel[1], pixel[2]));
    let linear_rgb = to_srgb(pixel).into_linear();

    let hsv = Hsv::from_color(linear_rgb);
    (hsv.saturation <= WHITE_SATURATION_THRESHOLD && hsv.value >= WHITE_VALUE_THRESHOLD)
        || (max_pixel > WHITE_BRIGHTNESS_THRESHOLD)
}
pub fn detect_color(img: &DynamicImage, bbox: &BoundingBox) -> Color {
    let mut color_count = HashMap::new();

    for (x, y) in bbox.iter() {
        let color: Color = get_color_lab(img.get_pixel(x, y));
        *color_count.entry(color).or_insert(0) += 1;
    }

    let max = color_count
        .into_iter()
        .max_by_key(|color| color.1)
        .expect("No maximum color found!");
    max.0
}

fn get_color_lab(pixel: Rgba<u8>) -> Color {
    if is_white(pixel) {
        Color::White
    } else {
        let distances: Vec<(i32, Color)> = colors()
            .iter()
            .map(|color| {
                let rgb = color.lab_rgb();
                let s_pixel = to_srgb(pixel);

                let lab: Lab = Lab::from_color(s_pixel);

                // Compute euclidean distance
                let distance = lab.distance_squared(rgb) as i32;
                (distance, *color)
            })
            .collect();
        // Minimum distance is the detected color
        let min_distance = distances
            .iter()
            .min_by_key(|(distance, _)| *distance)
            .expect("No maximum color found!");

        min_distance.1
    }
}

fn to_srgb(pixel: Rgba<u8>) -> palette::rgb::Rgb {
    Srgb::new(
        pixel.0[0] as f32 / 255.0,
        pixel.0[1] as f32 / 255.0,
        pixel.0[2] as f32 / 255.0,
    )
}
