use glam::Vec2;
use palette::Srgb;

use crate::helpers::min_max_points;

pub type Point2 = Vec2;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(&self, point: Point2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn rotate(&mut self, theta: f32) {
        let center = self.center();
        let mut points = self.points();
        for point in &mut points {
            *point -= center;
            *point = Vec2::from_angle(theta).rotate(*point);
            *point += center;
        }
        let (min_x, max_x, min_y, max_y) = min_max_points(&points);
        self.x = min_x;
        self.y = min_y;
        self.width = max_x - min_x;
        self.height = max_y - min_y;
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn points(&self) -> [Vec2; 4] {
        [
            Vec2::new(self.x, self.y),
            Vec2::new(self.x + self.width, self.y),
            Vec2::new(self.x + self.width, self.y + self.height),
            Vec2::new(self.x, self.y + self.height),
        ]
    }
}

pub struct Sprite {
    pub rect: Rect,
    pub texture: Vec<u8>,
}

impl Sprite {
    pub fn new(rect: Rect, texture: Vec<u8>) -> Self {
        Self { rect, texture }
    }
}

pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }
}

pub struct ColorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Srgb,
}

impl ColorRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Srgb) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
        }
    }
}

pub struct TextureRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub texture: Vec<u8>,
    pub texture_width: usize,
    pub texture_height: usize,
}

impl TextureRect {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        texture: Vec<u8>,
        texture_width: usize,
        texture_height: usize,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            texture,
            texture_width,
            texture_height,
        }
    }
}

pub struct GradientRect {
    pub rect: Rect,
    pub color_start: Srgb,
    pub color_end: Srgb,
}

impl GradientRect {
    pub fn new(rect: Rect, color_start: Srgb, color_end: Srgb) -> Self {
        Self {
            rect,
            color_start,
            color_end,
        }
    }
}

pub struct ColorLine {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Srgb,
}

impl ColorLine {
    pub fn new(start: Vec2, end: Vec2, color: Srgb) -> Self {
        Self { start, end, color }
    }
}

pub struct Circle {
    pub center: Point2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: Point2, radius: f32) -> Self {
        Self { center, radius }
    }
}

pub struct ColorCircle {
    pub center: Point2,
    pub radius: f32,
    pub color: Srgb,
}

impl ColorCircle {
    pub fn new(center: Point2, radius: f32, color: Srgb) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }
}
