use glam::Vec2;
use palette::Srgb;

use crate::{
    map::{MAP_HEIGHT, MAP_WIDTH},
    WIDTH,
};

pub fn clear_frame(frame: &mut [u8], color: Srgb) {
    for i in 0..frame.len() / 4 {
        frame[i * 4] = (color.red * 255.0) as u8;
        frame[i * 4 + 1] = (color.green * 255.0) as u8;
        frame[i * 4 + 2] = (color.blue * 255.0) as u8;
        frame[i * 4 + 3] = 255;
    }
}

pub fn frame_to_rgb(frame: &[u8]) -> Vec<Srgb> {
    frame
        .chunks_exact(4)
        .map(|chunk| {
            Srgb::new(
                chunk[0] as f32 / 255.0,
                chunk[1] as f32 / 255.0,
                chunk[2] as f32 / 255.0,
            )
        })
        .collect()
}

pub fn rgb_to_frame(rgb: &[Srgb]) -> Vec<u8> {
    rgb.iter()
        .flat_map(|color| {
            vec![
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                255,
            ]
        })
        .collect()
}

pub fn min_max_points(points: &[Vec2]) -> (f32, f32, f32, f32) {
    let mut min_x = std::f32::MAX;
    let mut max_x = std::f32::MIN;
    let mut min_y = std::f32::MAX;
    let mut max_y = std::f32::MIN;
    for point in points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    (min_x, max_x, min_y, max_y)
}

pub fn draw_pixel_brightness_rgb(
    screen: &mut [Srgb],
    x: f32,
    y: f32,
    color: Srgb,
    brightness: f32,
) {
    if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < WIDTH as f32 {
        let index = x as usize + y as usize * WIDTH as usize;
        if index < screen.len() {
            screen[index] = Srgb::new(
                screen[index].red + color.red * brightness,
                screen[index].green + color.green * brightness,
                screen[index].blue + color.blue * brightness,
            );
        }
    }
}

pub fn draw_pixel_rgb(screen: &mut [Srgb], x: f32, y: f32, color: Srgb) {
    if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < WIDTH as f32 {
        let index = x as usize + y as usize * WIDTH as usize;
        if index < screen.len() {
            screen[index] = color;
        }
    }
}

pub fn is_in_circle(p: Vec2, center: Vec2, radius: f32) -> bool {
    (p.x - center.x).powi(2) + (p.y - center.y).powi(2) < radius.powi(2)
}

pub fn draw_pixel_raw(screen: &mut [u8], x: f32, y: f32, color: Srgb) {
    if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < WIDTH as f32 {
        let index = x as usize + y as usize * WIDTH as usize;
        if (index * 4 + 2) < screen.len() {
            screen[index * 4] = (color.red * 255.0) as u8;
            screen[index * 4 + 1] = (color.green * 255.0) as u8;
            screen[index * 4 + 2] = (color.blue * 255.0) as u8;
            screen[index * 4 + 3] = 255;
        }
    }
}

pub fn draw_pixel_brightness_raw(screen: &mut [u8], x: f32, y: f32, color: Srgb, brightness: f32) {
    if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < WIDTH as f32 {
        let index = x as usize + y as usize * WIDTH as usize;
        if index < screen.len() {
            screen[index * 4] = (((screen[index * 4] as f32 / 255.0) * (1.0 - brightness)
                + color.red * brightness)
                .min(1.0)
                .max(0.0)
                * 255.0) as u8;
            screen[index * 4 + 1] = (((screen[index * 4 + 1] as f32 / 255.0) * (1.0 - brightness)
                + color.green * brightness)
                .min(1.0)
                .max(0.0)
                * 255.0) as u8;
            screen[index * 4 + 2] = (((screen[index * 4 + 2] as f32 / 255.0) * (1.0 - brightness)
                + color.blue * brightness)
                .min(1.0)
                .max(0.0)
                * 255.0) as u8;
            screen[index * 4 + 3] = 255;
        }
    }
}

pub fn screen_to_map(x: f32, y: f32) -> (usize, usize) {
    let x = x / WIDTH as f32 * MAP_WIDTH as f32;
    let y = y / WIDTH as f32 * MAP_HEIGHT as f32;
    (x as usize, y as usize)
}
