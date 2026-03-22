//! Rendering logic and GPU command execution

use crate::common::error::AppResult;
use crate::visualization::Plugin;

/// Renderer for handling GPU rendering operations
pub struct Renderer {
    pub egui_renderer: egui_wgpu::Renderer,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let egui_renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1);
        Self { egui_renderer }
    }

    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plugins: &[Plugin],
        plugin_index: usize,
        bind_group: &wgpu::BindGroup,
        particle_render_pipeline: &wgpu::RenderPipeline,
        quad_buffer: &wgpu::Buffer,
        particle_buffer: &wgpu::Buffer,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_desc: &egui_wgpu::ScreenDescriptor,
        textures_delta: &egui::TexturesDelta,
    ) -> AppResult<()> {
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update egui textures
        for (id, image_delta) in &textures_delta.set {
            self.egui_renderer.update_texture(device, queue, *id, image_delta);
        }

        // Compute particles - TODO: This should be handled separately

        // Update egui vertex/index buffers
        self.egui_renderer.update_buffers(device, queue, &mut encoder, paint_jobs, screen_desc);

        // Render pass
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Visualization plugin
            rpass.set_pipeline(&plugins[plugin_index].render_pipeline);
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..3, 0..1);

            // Particles
            rpass.set_pipeline(particle_render_pipeline);
            rpass.set_vertex_buffer(0, quad_buffer.slice(..));
            rpass.set_vertex_buffer(1, particle_buffer.slice(..));
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..4, 0..crate::config::constants::NUM_PARTICLES);

            // egui overlay
            self.egui_renderer.render(&mut rpass, paint_jobs, screen_desc);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Free egui textures
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        Ok(())
    }
}
