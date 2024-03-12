#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![allow(dead_code, unused_assignments)]

use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use std::time::Duration;

use egui::mutex::Mutex;
use error_iter::ErrorIter as _;
use fps_ticker::Fps;
use game_loop::{game_loop, Time, TimeTrait};
use glam::Vec2;

use log::error;
use palette::Srgb;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod geo;
mod graphics;
mod gui;
mod helpers;
mod map;

use geo::*;
use graphics::*;
use gui::*;
use helpers::*;
use map::*;

const WIDTH: u32 = 1920 / 3;
const HEIGHT: u32 = 1080 / 3;

#[derive(Debug, Default)]
struct Controls {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
}

struct Game {
    pub pixels: Pixels,
    pub world: Arc<Mutex<World>>,
    pub controls: Controls,
    pub input: WinitInputHelper,
    pub paused: bool,
    pub framework: Framework,
    pub fps: Arc<Mutex<Fps>>,
}

impl Game {
    pub fn new(
        pixels: Pixels,
        framework: Framework,
        fps: Arc<Mutex<Fps>>,
        world: Arc<Mutex<World>>,
    ) -> Self {
        Self {
            pixels,
            world,
            controls: Controls::default(),
            input: WinitInputHelper::new(),
            paused: false,
            framework,
            fps,
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);
        WindowBuilder::new()
            .with_title("wolfenstein-rs")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let fps = Arc::new(Mutex::new(Fps::default()));

    let mut textures: Vec<Vec<Vec<u8>>> = vec![];
    let mut r = std::io::Cursor::new(include_bytes!("../assets/textures.png"));
    let img = image::load(&mut r, image::ImageFormat::Png).unwrap();
    for i in 0..=7 {
        let img = img.crop_imm(i * (img.width() / 8), 0, img.width() / 8, img.height());
        let mut vertical_chunks = vec![];
        for i in 0..img.width() {
            let chunk = img.crop_imm(i, 0, 1, img.height()).to_rgba8().to_vec();
            vertical_chunks.push(chunk);
        }
        textures.push(vertical_chunks);
    }

    // Write the textures to disk for debugging
    // for (i, texture) in textures.iter().enumerate() {
    //     for (j, texture) in texture.iter().enumerate() {
    //         texture
    //             .save(format!("textures/texture_{}_{}.png", i, j))
    //             .unwrap();
    //     }
    // }

    let world = Arc::new(Mutex::new(World::new(textures.clone())));

    let (pixels, framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
            fps.clone(),
            world.clone(),
        );

        (pixels, framework)
    };

    let game = Game::new(pixels, framework, fps.clone(), world.clone());
    game_loop(
        event_loop,
        window,
        game,
        60,
        0.1,
        |g| {
            // Update
            g.game.world.lock().update(&g.game.controls);
        },
        |g| {
            // Draw
            {
                g.game.fps.lock().tick();
            }
            {
                // let s = Stopwatch::start_new();
                g.game.world.lock().draw(g.game.pixels.frame_mut());
                // println!("draw: {}", s.elapsed_ms());
            }
            g.game.framework.prepare(&g.window);

            let render_result = g
                .game
                .pixels
                .render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    g.game.framework.render(encoder, render_target, context);

                    Ok(())
                });

            // Basic error handling
            if let Err(err) = render_result {
                log_error("pixels.render", err);
                g.exit();
            }
            let time_step = Duration::from_secs_f64(1.0 / 144.0);
            let dt = time_step.as_secs_f64() - Time::now().sub(&g.current_instant());
            if dt > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(dt));
            }
        },
        |g, event| {
            // Handle events
            if g.game.input.update(&event) {
                // Close events
                if g.game.input.key_pressed(VirtualKeyCode::Escape)
                    || g.game.input.close_requested()
                {
                    g.exit();
                }
                g.game.controls = {
                    let mut controls = Controls::default();
                    if g.game.input.key_held(VirtualKeyCode::W) {
                        controls.forward = true;
                    }
                    if g.game.input.key_held(VirtualKeyCode::S) {
                        controls.backward = true;
                    }
                    if g.game.input.key_held(VirtualKeyCode::A) {
                        controls.strafe_left = true;
                    }
                    if g.game.input.key_held(VirtualKeyCode::D) {
                        controls.strafe_right = true;
                    }
                    if g.game.input.key_held(VirtualKeyCode::Q) {
                        controls.left = true;
                    }
                    if g.game.input.key_held(VirtualKeyCode::E) {
                        controls.right = true;
                    }
                    controls
                };

                // Update the scale factor
                if let Some(scale_factor) = g.game.input.scale_factor() {
                    g.game.framework.scale_factor(scale_factor);
                }

                // // Resize the window
                // if let Some(size) = g.game.input.window_resized() {
                //     if let Err(err) = g.game.pixels.resize_surface(size.width, size.height) {
                //         log_error("pixels.resize_surface", err);
                //         g.exit();
                //     }
                //     g.game.framework.resize(size.width, size.height);
                // }
            }

            match event {
                Event::WindowEvent { event, .. } => {
                    // Update egui inputs
                    g.game.framework.handle_event(&event);
                }
                _ => (),
            }
        },
    );
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

struct World {
    speed: f32,
    rotation_speed: f32,
    x: f32,
    y: f32,
    theta: f32,
    fov: f32,
    textures: Vec<Vec<Vec<u8>>>,
}

impl World {
    fn new(textures: Vec<Vec<Vec<u8>>>) -> Self {
        Self {
            x: WIDTH as f32 / 2.0,
            y: HEIGHT as f32 / 2.0,
            theta: -PI / 2.0,
            speed: 2.0,
            rotation_speed: 0.05,
            fov: 60.0,
            textures,
        }
    }

    fn will_hit_obstacle(&self, x: f32, y: f32) -> Option<u8> {
        let map_x = (x / (WIDTH as f32 / MAP_WIDTH as f32)).floor() as usize;
        let map_y = (y / (HEIGHT as f32 / MAP_HEIGHT as f32)).floor() as usize;
        if map_x >= MAP_WIDTH || map_y >= MAP_HEIGHT {
            return Some(0);
        }
        if MAP[map_y][map_x] != 0 {
            return Some(MAP[map_y][map_x]);
        } else {
            return None;
        }
    }

    fn ray_hits(&self, start: Point2, angle: f32) -> Option<(Vec<Vec<u8>>, Point2, u32)> {
        let mut x = start.x;
        let mut y = start.y;
        let dx = angle.cos();
        let dy = angle.sin();
        let mut step_x = 1;
        let mut step_y = 1;
        let mut side = 0;
        let mut side_dist_x = 0.0;
        let mut side_dist_y = 0.0;
        let delta_dist_x = (1.0 / dx).abs();
        let delta_dist_y = (1.0 / dy).abs();
        let mut hit = None;
        if dx < 0.0 {
            step_x = -1;
            side_dist_x = (x - (x as i32 as f32)) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = ((x as i32 as f32) + 1.0 - x) * delta_dist_x;
        }
        if dy < 0.0 {
            step_y = -1;
            side_dist_y = (y - (y as i32 as f32)) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = ((y as i32 as f32) + 1.0 - y) * delta_dist_y;
        }
        while hit.is_none() {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                x += step_x as f32;
                side = 0;
            } else {
                side_dist_y += delta_dist_y;
                y += step_y as f32;
                side = 1;
            }
            let map_x = (x / (WIDTH as f32 / MAP_WIDTH as f32)).floor() as usize;
            let map_y = (y / (HEIGHT as f32 / MAP_HEIGHT as f32)).floor() as usize;
            if map_x >= MAP_WIDTH || map_y >= MAP_HEIGHT {
                return None;
            }
            if MAP[map_y][map_x] != 0 {
                let texture = self.textures[MAP[map_y][map_x] as usize - 1].clone();
                hit = Some((texture, Point2::new(x, y), side));
            }
        }
        hit
    }

    fn update(&mut self, controls: &Controls) {
        if controls.forward {
            let new_x = self.theta.cos() * self.speed;
            let new_y = self.theta.sin() * self.speed;
            if self
                .will_hit_obstacle(self.x + new_x, self.y + new_y)
                .is_none()
            {
                self.x += new_x;
                self.y += new_y;
            }
        }
        if controls.backward {
            let new_x = -self.theta.cos() * self.speed;
            let new_y = -self.theta.sin() * self.speed;
            if self
                .will_hit_obstacle(self.x + new_x, self.y + new_y)
                .is_none()
            {
                self.x += new_x;
                self.y += new_y;
            }
        }
        if controls.strafe_right {
            let new_x = (self.theta + TAU / 4.0).cos() * self.speed;
            let new_y = (self.theta + TAU / 4.0).sin() * self.speed;
            if self
                .will_hit_obstacle(self.x + new_x, self.y + new_y)
                .is_none()
            {
                self.x += new_x;
                self.y += new_y;
            }
        }
        if controls.strafe_left {
            let new_x = (self.theta - TAU / 4.0).cos() * self.speed;
            let new_y = (self.theta - TAU / 4.0).sin() * self.speed;
            if self
                .will_hit_obstacle(self.x + new_x, self.y + new_y)
                .is_none()
            {
                self.x += new_x;
                self.y += new_y;
            }
        }
        if controls.right {
            self.theta += self.rotation_speed;
        }
        if controls.left {
            self.theta -= self.rotation_speed;
        }
        self.theta = self.theta % TAU;
    }

    fn draw_minimap(&self, frame: &mut [u8]) {
        let cell_size_x = WIDTH as f32 / MAP_WIDTH as f32;
        let cell_size_y = HEIGHT as f32 / MAP_HEIGHT as f32;
        for i in 0..MAP_HEIGHT {
            for j in 0..MAP_WIDTH {
                let color = match MAP[i][j] {
                    0 => Srgb::new(0.0, 0.0, 0.0),
                    1 => Srgb::new(0.0, 0.0, 1.0),
                    2 => Srgb::new(0.0, 1.0, 0.0),
                    3 => Srgb::new(1.0, 0.0, 0.0),
                    4 => Srgb::new(1.0, 1.0, 0.0),
                    5 => Srgb::new(1.0, 0.0, 1.0),
                    _ => Srgb::new(1.0, 1.0, 1.0),
                };

                ColorRect::new(
                    j as f32 * cell_size_x,
                    i as f32 * cell_size_y,
                    cell_size_x,
                    cell_size_y,
                    color,
                )
                .draw(frame);
            }
        }
    }

    fn draw_player(&self, frame: &mut [u8]) {
        ColorRect::new(
            self.x - 2.0,
            self.y - 2.0,
            4.0,
            4.0,
            Srgb::new(1.0, 1.0, 1.0),
        )
        .draw(frame);
    }

    fn draw_rays(&self, frame: &mut [u8]) {
        let theta_step = self.fov / WIDTH as f32;
        let tile_width = WIDTH as f32 / MAP_WIDTH as f32;
        let tile_ratio = 64.0 / tile_width;

        for i in 0..WIDTH {
            if let Some((columns, hit, side)) = self.ray_hits(
                Vec2::new(self.x, self.y),
                self.theta + (i as f32 - WIDTH as f32 / 2.0) * theta_step.to_radians(),
            ) {
                let dist = (Vec2::new(self.x, self.y) - hit).length();
                let height = HEIGHT as f32 / dist * 50.0;

                let tile_relative_x = if side == 1 { hit.x } else { hit.y } % tile_width;
                let column_index = ((tile_relative_x * tile_ratio) as usize).min(63);

                let texture = &columns[column_index];
                TextureRect::new(
                    i as f32,
                    (HEIGHT as f32 - height) / 2.0,
                    1.0,
                    height,
                    texture.to_vec(),
                    1,
                    64,
                )
                .draw(frame);
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        clear_frame(frame, Srgb::new(0.0, 0.0, 0.0));
        self.draw_rays(frame);
        // self.draw_minimap(frame);
        // self.draw_player(frame);
    }
}
