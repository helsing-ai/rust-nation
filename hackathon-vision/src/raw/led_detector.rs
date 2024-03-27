use crate::raw::preprocessor::BrightArea;
use crate::raw::{
    bounding_box::BoundingBox, colors::detect_color, colors::Color,
    preprocessor::extract_bright_areas, utils::bbox_resize,
};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Luma};

#[derive(Debug, Clone)]
pub struct Led {
    pub bbox: BoundingBox,
    pub color: Color,
}

pub struct LedDetectionConfig {
    pub width: u32,
    pub height: u32,
    pub filter: FilterType,
    pub radius_1: f32,
    pub radius_2: f32,
    pub threshold_value: u8,
    pub min_size: (u32, u32),
    pub max_size: (u32, u32),
}

impl Default for LedDetectionConfig {
    fn default() -> Self {
        Self {
            // According to last drone pics
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        }
    }
}
fn find_leds_areas(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> eyre::Result<Vec<BoundingBox>> {
    let mut visited: Vec<Vec<bool>> =
        vec![vec![false; img.height() as usize]; img.width() as usize];
    let mut bounding_boxes = Vec::new();

    for (x, y, pixel) in img.enumerate_pixels() {
        if pixel[0] == 0 && !visited[x as usize][y as usize] {
            // Found a new island, perform flood fill and calculate bounding box
            let mut stack = vec![(x, y)];
            let mut bbox = BoundingBox::new(x, y, x, y)?;

            while let Some((cx, cy)) = stack.pop() {
                if cx < img.width()
                    && cy < img.height()
                    && img.get_pixel(cx, cy)[0] == 0
                    && !visited[cx as usize][cy as usize]
                {
                    // Update bounding box
                    let x_min = bbox.x_min().min(cx);
                    let y_min = bbox.y_min().min(cy);
                    let x_max = bbox.x_max().max(cx);
                    let y_max = bbox.y_max().max(cy);
                    bbox.set_coordinates(x_min, y_min, x_max, y_max)?;

                    // Add neighboring pixels to the stack if they are within the image bounds
                    if cx.saturating_sub(1) > 0 {
                        stack.push((cx - 1, cy));
                    } // left
                    if cx + 1 < img.width() {
                        stack.push((cx + 1, cy));
                    } // right
                    if cy.saturating_sub(1) > 0 {
                        stack.push((cx, cy - 1));
                    } // up
                    if cy + 1 < img.height() {
                        stack.push((cx, cy + 1));
                    } // down
                    visited[cx as usize][cy as usize] = true;
                }
            }

            bounding_boxes.push(bbox);
        }
    }
    Ok(bounding_boxes)
}

pub fn get_leds(image: &DynamicImage, config: &LedDetectionConfig) -> eyre::Result<Vec<Led>> {
    let BrightArea {
        thresholded,
        resized,
    } = extract_bright_areas(image, config)?;

    // Use counting-islands algorithm to find islands of "very bright" pixels
    let bounding_boxes = find_leds_areas(&thresholded)?;

    // For each bounding box, detect color
    let bounding_boxes_with_color: Vec<Led> = bounding_boxes
        .clone()
        .into_iter()
        .filter_map(|bbox| {
            if !bbox.is_within_size_bounds(config.min_size, config.max_size) {
                None
            } else {
                let color = detect_color(&resized, &bbox);
                if let Ok(bbox_on_original_image) =
                    bbox_resize(&bbox, &image.dimensions(), &resized.dimensions())
                {
                    Some(Ok(Led {
                        bbox: bbox_on_original_image,
                        color,
                    }))
                } else {
                    None
                }
            }
        })
        .collect::<eyre::Result<Vec<Led>>>()?;
    Ok(bounding_boxes_with_color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance;
    use crate::raw::{bounding_box, utils::draw_bounding_boxes, BLUE, GREEN, RED, WHITE};
    use image::{imageops::FilterType, Rgba};
    use std::env;
    use std::path::{Path, PathBuf};

    fn create_new_path(path: &Path, suffix: &str) -> PathBuf {
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let ext = path.extension().unwrap().to_str().unwrap();
        let new_path = format!("{}_{}.{}", file_stem, suffix, ext);
        path.with_file_name(new_path)
    }

    fn test_image_overlay(path: PathBuf, config: LedDetectionConfig) -> Vec<Led> {
        let mut img = image::open(path.clone()).unwrap();

        let mut red_boxes = vec![];
        let mut blue_boxes = vec![];
        let mut green_boxes = vec![];
        let mut white_boxes = vec![];

        let leds = get_leds(&img, &config).unwrap();

        leds.iter().for_each(|led| match led.color {
            Color::Red => red_boxes.push(led.bbox),
            Color::Green => green_boxes.push(led.bbox),
            Color::Blue => blue_boxes.push(led.bbox),
            Color::White => white_boxes.push(led.bbox),
            Color::Unknown => {}
        });

        // For debug purposes, overlay bounding box with corresponding color
        draw_bounding_boxes(&mut img, red_boxes, Rgba(RED));
        draw_bounding_boxes(&mut img, blue_boxes, Rgba(BLUE));
        draw_bounding_boxes(&mut img, green_boxes, Rgba(GREEN));
        draw_bounding_boxes(&mut img, white_boxes, Rgba(WHITE));
        let overlay_color_path = create_new_path(&path, "overlay_color");
        img.save(overlay_color_path).unwrap();

        let bounding_boxes: Vec<BoundingBox> =
            leds.clone().into_iter().map(|led| led.bbox).collect();

        // Overlay all bounding boxes onto original image
        draw_bounding_boxes(&mut img, bounding_boxes, Rgba([255, 255, 255, 255]));
        let overlay_path = create_new_path(&path, "overlay");
        img.save(overlay_path).unwrap();

        leds
    }
    #[test]
    fn test_multiple_bulbs() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/test_multiple_bulbs.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 20,
            min_size: (4, 4),
            max_size: (40, 40),
        };
        let _ = test_image_overlay(path, config);
    }

    #[test]
    fn test_with_enclosure() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/test_with_enclosure.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 20,
            min_size: (6, 6),
            max_size: (25, 25),
        };
        let leds = test_image_overlay(path, config);
        let blue_leds: Vec<Led> = leds
            .clone()
            .into_iter()
            .filter(|led| led.color == Color::Blue)
            .collect();
        assert_eq!(blue_leds.len(), 1);
        let blue_led = &blue_leds[0];
        assert_eq!(blue_led.bbox.x_min(), 520);
        assert_eq!(blue_led.bbox.y_min(), 700);
        assert_eq!(blue_led.bbox.x_max(), 558);
        assert_eq!(blue_led.bbox.y_max(), 738);
    }

    #[test]
    pub fn colours_higher_up() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/all_colours_higher_up.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 12,
            min_size: (3, 3),
            max_size: (10, 10),
        };
        let leds = test_image_overlay(path, config);
    }

    #[test]
    pub fn colours_higher_up_2() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/all_colours_higher_up_2.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 15.0,
            threshold_value: 10,
            min_size: (3, 3),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
    }

    #[test]
    pub fn colours_horizontal() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/all_colours_horizontal.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 15.0,
            threshold_value: 10,
            min_size: (5, 5),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
    }

    #[test]
    pub fn all_colours_vertical() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/all_colours_vertical.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 15.0,
            threshold_value: 10,
            min_size: (5, 5),
            max_size: (25, 25),
        };
        let leds = test_image_overlay(path, config);
    }

    #[test]
    pub fn aerial_green() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/cropped_green_aerial.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 2.0,
            radius_2: 8.0,
            threshold_value: 12,
            min_size: (3, 3),
            max_size: (10, 10),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::Green);
        assert_eq!(led.bbox.x_min(), 1077);
        assert_eq!(led.bbox.y_min(), 994);
        assert_eq!(led.bbox.x_max(), 1106);
        assert_eq!(led.bbox.y_max(), 1023);
    }

    #[test]
    pub fn blue_800() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/blue_800.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 15,
            min_size: (10, 10),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.bbox.x_min(), 1146);
        assert_eq!(led.bbox.y_min(), 601);
        assert_eq!(led.bbox.x_max(), 1213);
        assert_eq!(led.bbox.y_max(), 672);
    }

    #[test]
    pub fn blue_7000() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/blue_7000.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 15,
            min_size: (10, 10),
            max_size: (40, 40),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.bbox.x_min(), 1175);
        assert_eq!(led.bbox.y_min(), 186);
        assert_eq!(led.bbox.x_max(), 1261);
        assert_eq!(led.bbox.y_max(), 276);
    }

    #[test]
    pub fn green_4000() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/green_4000.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (20, 10),
            max_size: (40, 40),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let green_led = &leds[0];
        assert_eq!(green_led.color, Color::Green);
        assert_eq!(green_led.bbox.x_min(), 991);
        assert_eq!(green_led.bbox.y_min(), 331);
        assert_eq!(green_led.bbox.x_max(), 1090);
        assert_eq!(green_led.bbox.y_max(), 416);
    }

    #[test]
    pub fn green_20000() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/new_leds/green_20000.png");
        let config = LedDetectionConfig {
            width: 400,
            height: 400,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 15,
            min_size: (10, 10),
            max_size: (40, 40),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let green_led = &leds[0];
        assert_eq!(green_led.color, Color::Green);
        assert_eq!(green_led.bbox.x_min(), 1255);
        assert_eq!(green_led.bbox.y_min(), 575);
        assert_eq!(green_led.bbox.x_max(), 1326);
        assert_eq!(green_led.bbox.y_max(), 660);
    }

    #[test]
    pub fn red_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/red.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::Red);
        assert_eq!(led.bbox.x_min(), 1014);
        assert_eq!(led.bbox.y_min(), 729);
        assert_eq!(led.bbox.x_max(), 1039);
        assert_eq!(led.bbox.y_max(), 752);
    }

    #[test]
    pub fn blue_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/blue.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::Blue);
        assert_eq!(led.bbox.x_min(), 1010);
        assert_eq!(led.bbox.y_min(), 646);
        assert_eq!(led.bbox.x_max(), 1035);
        assert_eq!(led.bbox.y_max(), 671);
    }

    #[test]
    pub fn green_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/green.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::Green);
        assert_eq!(led.bbox.x_min(), 737);
        assert_eq!(led.bbox.y_min(), 436);
        assert_eq!(led.bbox.x_max(), 764);
        assert_eq!(led.bbox.y_max(), 463);
    }

    #[test]
    pub fn green_2_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/green_2.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::Green);
        assert_eq!(led.bbox.x_min(), 854);
        assert_eq!(led.bbox.y_min(), 556);
        assert_eq!(led.bbox.x_max(), 883);
        assert_eq!(led.bbox.y_max(), 586);
    }
    #[test]
    pub fn warm_white_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/warm_white.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::White);
        assert_eq!(led.bbox.x_min(), 966);
        assert_eq!(led.bbox.y_min(), 694);
        assert_eq!(led.bbox.x_max(), 995);
        assert_eq!(led.bbox.y_max(), 723);
    }

    #[test]
    pub fn warm_white_2_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/warm_white_2.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 1);
        let led = &leds[0];
        assert_eq!(led.color, Color::White);
        assert_eq!(led.bbox.x_min(), 995);
        assert_eq!(led.bbox.y_min(), 561);
        assert_eq!(led.bbox.x_max(), 1023);
        assert_eq!(led.bbox.y_max(), 588);
    }

    #[test]
    pub fn both_white_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/both_white.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 2);
        let led = &leds[0];
        let led_2 = &leds[1];
        assert_eq!(led.color, Color::White);
        assert_eq!(led_2.color, Color::White);
    }

    #[test]
    pub fn blue_green_red_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/blue_green_red.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 3);
        let mut blue_led = leds[0].clone();
        let mut red_led = leds[0].clone();
        let mut green_led = leds[0].clone();
        for led in leds {
            match led.color {
                Color::Red => red_led = led,
                Color::Green => green_led = led,
                Color::Blue => blue_led = led,
                _ => {}
            }
        }

        let b_g = distance(&blue_led, &green_led);
        let g_r = distance(&green_led, &red_led);
        let r_b = distance(&red_led, &blue_led);

        assert_eq!(blue_led.bbox.x_min(), 625);
        assert_eq!(blue_led.bbox.y_min(), 342);
        assert_eq!(blue_led.bbox.x_max(), 650);
        assert_eq!(blue_led.bbox.y_max(), 367);

        assert_eq!(green_led.bbox.x_min(), 927);
        assert_eq!(green_led.bbox.y_min(), 796);
        assert_eq!(green_led.bbox.x_max(), 952);
        assert_eq!(green_led.bbox.y_max(), 823);

        assert_eq!(red_led.bbox.x_min(), 392);
        assert_eq!(red_led.bbox.y_min(), 765);
        assert_eq!(red_led.bbox.x_max(), 406);
        assert_eq!(red_led.bbox.y_max(), 779);

        assert_eq!(b_g, 546);
        assert_eq!(r_b, 481);
        assert_eq!(g_r, 541);
    }

    #[test]
    pub fn green_blue_red_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/green_blue_red.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 3);
        let mut blue_led = leds[0].clone();
        let mut red_led = leds[0].clone();
        let mut green_led = leds[0].clone();
        for led in leds {
            match led.color {
                Color::Red => red_led = led,
                Color::Green => green_led = led,
                Color::Blue => blue_led = led,
                _ => {}
            }
        }

        let b_g = distance(&blue_led, &green_led);
        let g_r = distance(&green_led, &red_led);
        let r_b = distance(&red_led, &blue_led);

        assert_eq!(blue_led.bbox.x_min(), 796);
        assert_eq!(blue_led.bbox.y_min(), 392);
        assert_eq!(blue_led.bbox.x_max(), 821);
        assert_eq!(blue_led.bbox.y_max(), 417);

        assert_eq!(green_led.bbox.x_min(), 1104);
        assert_eq!(green_led.bbox.y_min(), 825);
        assert_eq!(green_led.bbox.x_max(), 1129);
        assert_eq!(green_led.bbox.y_max(), 850);

        assert_eq!(red_led.bbox.x_min(), 571);
        assert_eq!(red_led.bbox.y_min(), 819);
        assert_eq!(red_led.bbox.x_max(), 589);
        assert_eq!(red_led.bbox.y_max(), 838);

        assert_eq!(b_g, 531);
        assert_eq!(r_b, 481);
        assert_eq!(g_r, 536);
    }

    #[test]
    pub fn red_green_from_drone() {
        let path = env::current_dir()
            .unwrap()
            .join("../resources/from_drone/red_green.png");
        let config = LedDetectionConfig {
            width: 800,
            height: 800,
            filter: FilterType::Gaussian,
            radius_1: 4.0,
            radius_2: 8.0,
            threshold_value: 10,
            min_size: (7, 7),
            max_size: (20, 20),
        };
        let leds = test_image_overlay(path, config);
        assert_eq!(leds.len(), 2);
        let mut red_led = leds[0].clone();
        let mut green_led = leds[0].clone();
        for led in leds {
            match led.color {
                Color::Red => red_led = led,
                Color::Green => green_led = led,
                _ => {}
            }
        }
        let g_r = distance(&green_led, &red_led);

        assert_eq!(green_led.bbox.x_min(), 1131);
        assert_eq!(green_led.bbox.y_min(), 423);
        assert_eq!(green_led.bbox.x_max(), 1160);
        assert_eq!(green_led.bbox.y_max(), 452);

        assert_eq!(red_led.bbox.x_min(), 602);
        assert_eq!(red_led.bbox.y_min(), 419);
        assert_eq!(red_led.bbox.x_max(), 625);
        assert_eq!(red_led.bbox.y_max(), 440);

        assert_eq!(g_r, 532);
    }
}
