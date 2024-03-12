use std::sync::Arc;

use egui::mutex::Mutex;
use egui::{ClippedPrimitive, Context, DragValue, TexturesDelta};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use fps_ticker::Fps;
use pixels::{wgpu, PixelsContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::World;

pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,
    // State for the GUI
    gui: Gui,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        fps: Arc<Mutex<Fps>>,
        world: Arc<Mutex<World>>,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new(fps, world);

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

struct Gui {
    /// Only show the egui window when true.
    fps: Arc<Mutex<Fps>>,
    world: Arc<Mutex<World>>,
}

impl Gui {
    fn new(fps: Arc<Mutex<Fps>>, world: Arc<Mutex<World>>) -> Self {
        Self { fps, world }
    }

    fn ui(&mut self, ctx: &Context) {
        egui::Window::new("Settings").show(ctx, |ui| {
            ui.label(format!("FPS: {:.2}", self.fps.lock().avg()));
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(DragValue::new(&mut self.world.lock().speed).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Rot. speed:");
                ui.add(DragValue::new(&mut self.world.lock().rotation_speed).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(DragValue::new(&mut self.world.lock().x).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                ui.add(DragValue::new(&mut self.world.lock().y).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Angle:");
                ui.add(DragValue::new(&mut self.world.lock().theta).speed(0.01));
                ui.label(format!("cos {:.2}", self.world.lock().theta.cos()));
                ui.label(format!("sin {:.2}", self.world.lock().theta.sin()));
            });
            ui.horizontal(|ui| {
                ui.label("FOV:");
                ui.add(DragValue::new(&mut self.world.lock().fov).speed(0.1));
            });
        });
        // egui::Window::new("Help").show(ctx, |ui| {
        //     ui.label("Move: WASD");
        //     ui.label("Rotate: QE");
        // });
    }
}
