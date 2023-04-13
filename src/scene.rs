use iced_wgpu;
use iced_winit::Color;
use wgpu::RenderPipeline;

pub struct Scene {
    pipeline: RenderPipeline,
}

impl Scene {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Scene {
        let pipeline = build_pipeline(device, texture_format);
        Scene { pipeline }
    }

    pub fn clear<'a>(
        &self,
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
        background_color: Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                //Defining the pipeline operations
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..4, 0..1);
    }
}

fn build_pipeline(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> RenderPipeline {
    let (vs_module, fs_module) = (
        device.create_shader_module(wgpu::include_wgsl!("shader/vert.wgsl")),
        device.create_shader_module(wgpu::include_wgsl!("shader/vert.wgsl")),
    );
    todo!()
}