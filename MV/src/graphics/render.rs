//! GPU render functions

use crate::common::error::AppResult;
use crate::config::constants::*;

use super::GpuResources;

impl GpuResources {
    pub fn render(
        &mut self,
        plugin_index: usize,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_desc: &egui_wgpu::ScreenDescriptor,
        textures_delta: &egui::TexturesDelta,
    ) -> AppResult<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update egui textures
        for (id, image_delta) in &textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // Compute particles
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.particle_bind_group, &[]);
            cpass.dispatch_workgroups(NUM_PARTICLES / COMPUTE_WORKGROUP_SIZE + 1, 1, 1);
        }

        // Update egui vertex/index buffers
        self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, paint_jobs, screen_desc);

        // Collect references to avoid borrow conflicts inside the render pass block
        let plugin_pipeline = &self.plugins[plugin_index].render_pipeline;
        let bind_group = &self.bind_group;
        let particle_render_pipeline = &self.particle_render_pipeline;
        let quad_buffer = &self.buffers.quad_buffer;
        let particle_buffer = &self.buffers.particle_buffer;

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
            rpass.set_pipeline(plugin_pipeline);
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..3, 0..1);

            // Particles
            rpass.set_pipeline(particle_render_pipeline);
            rpass.set_vertex_buffer(0, quad_buffer.slice(..));
            rpass.set_vertex_buffer(1, particle_buffer.slice(..));
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..4, 0..NUM_PARTICLES);

            // egui overlay
            self.egui_renderer.render(&mut rpass, paint_jobs, screen_desc);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Free egui textures
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        Ok(())
    }
}
