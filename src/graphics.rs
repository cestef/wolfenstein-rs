use crate::{geo::*, helpers::*, HEIGHT, WIDTH};
use glam::Vec2;
use palette::Srgb;

pub trait Drawable {
    fn draw(&self, screen: &mut [u8]);
}

impl Drawable for ColorRect {
    fn draw(&self, screen: &mut [u8]) {
        let x_start = self.x.ceil() as usize;
        let x_end = (self.x + self.width).ceil() as usize;
        let y_start = self.y.ceil() as usize;
        let y_end = (self.y + self.height).ceil() as usize;

        let mut pixels = Vec::new();
        for i in x_start..x_end {
            for j in y_start..y_end {
                pixels.push((i, j, self.color));
            }
        }

        for (i, j, color) in pixels {
            draw_pixel_raw(screen, i as f32, j as f32, color);
        }
    }
}

impl Drawable for ColorLine {
    /// Xiaolin Wu's line algorithm using draw_pixel_brightness
    fn draw(&self, screen: &mut [u8]) {
        let (mut x0, mut y0) = (self.start.x, self.start.y);
        let (mut x1, mut y1) = (self.end.x, self.end.y);
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let gradient = dy / dx;
        let mut y = y0 + gradient;
        for x in (x0 as usize)..(x1 as usize) {
            if steep {
                draw_pixel_brightness_raw(screen, y, x as f32, self.color, 1.0 - y.fract());
                draw_pixel_brightness_raw(screen, y + 1.0, x as f32, self.color, y.fract());
            } else {
                draw_pixel_brightness_raw(screen, x as f32, y, self.color, 1.0 - y.fract());
                draw_pixel_brightness_raw(screen, x as f32, y + 1.0, self.color, y.fract());
            }
            y += gradient;
        }
    }
}

impl Drawable for Sprite {
    fn draw(&self, screen: &mut [u8]) {
        for i in 0..(self.rect.width as usize) {
            for j in 0..(self.rect.height as usize) {
                let index =
                    (self.rect.x as usize + i) + (self.rect.y as usize + j) * WIDTH as usize;
                let texture_index = i + j * self.rect.width as usize;
                let color = Srgb::new(
                    self.texture[texture_index * 4] as f32 / 255.0,
                    self.texture[texture_index * 4 + 1] as f32 / 255.0,
                    self.texture[texture_index * 4 + 2] as f32 / 255.0,
                );
                draw_pixel_raw(
                    screen,
                    index as f32 % WIDTH as f32,
                    index as f32 / WIDTH as f32,
                    color,
                );
            }
        }
    }
}

impl ColorCircle {
    fn draw_circle(&self, screen: &mut [u8], p: Vec2, color: Srgb) {
        let c = self.center;
        draw_pixel_raw(screen, c.x + p.x, c.y + p.y, color);
        draw_pixel_raw(screen, c.x - p.x, c.y + p.y, color);
        draw_pixel_raw(screen, c.x + p.x, c.y - p.y, color);
        draw_pixel_raw(screen, c.x - p.x, c.y - p.y, color);
        draw_pixel_raw(screen, c.x + p.y, c.y + p.x, color);
        draw_pixel_raw(screen, c.x - p.y, c.y + p.x, color);
        draw_pixel_raw(screen, c.x + p.y, c.y - p.x, color);
        draw_pixel_raw(screen, c.x - p.y, c.y - p.x, color);
    }
}

impl Drawable for ColorCircle {
    fn draw(&self, screen: &mut [u8]) {
        let mut x = 0.0;
        let mut y = self.radius;
        let mut d = 3.0 - 2.0 * self.radius;
        self.draw_circle(screen, Point2::new(x, y), self.color);
        while y >= x {
            x += 1.0;
            if d > 0.0 {
                y -= 1.0;
                d += 4.0 * (x - y) + 10.0;
            } else {
                d += 4.0 * x + 6.0;
            }
            self.draw_circle(screen, Point2::new(x, y), self.color);
        }
    }
}

impl Drawable for TextureRect {
    fn draw(&self, screen: &mut [u8]) {
        let x_start = self.x.max(0.0).ceil() as usize;
        let x_end = (self.x + self.width).min(WIDTH as f32).ceil() as usize;
        let y_start = self.y.max(0.0).ceil() as usize;
        let y_end = (self.y + self.height).min(HEIGHT as f32).ceil() as usize;

        let texture_width = self.texture_width as f32;
        let texture_height = self.texture_height as f32;

        for i in x_start..x_end {
            for j in y_start..y_end {
                let texture_x = ((i as f32 - self.x) / self.width) * texture_width;
                let texture_y = ((j as f32 - self.y) / self.height) * texture_height;

                let texture_index = (texture_x.floor() as usize
                    + texture_y.floor() as usize * self.texture_width)
                    * 4;

                let color = Srgb::new(
                    self.texture[texture_index] as f32 / 255.0,
                    self.texture[texture_index + 1] as f32 / 255.0,
                    self.texture[texture_index + 2] as f32 / 255.0,
                );
                draw_pixel_raw(screen, i as f32, j as f32, color);
            }
        }
    }
}
