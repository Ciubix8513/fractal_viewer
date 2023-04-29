use bytemuck::{Pod, Zeroable};
use iced_wgpu::wgpu::{self, util::DeviceExt, BindGroup, Buffer};
use wgpu::RenderPipeline;

//Make memory layout the same as in C
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct ShaderDataUniforms {
    pub position: [f32; 2],
    pub resolution: [u32; 2],
    pub aspect: f32,
    pub dummy: f32,
    pub zoom: f32,
    pub arr_len: u32,
    pub fractal: u32,
    pub max_iter: u32,
    pub num_colors: u32,
    pub msaa: u32,
}
impl ShaderDataUniforms {
    pub fn to_uniform_data(self) -> [u32; 12] {
        [
            self.position[0].to_bits(),
            self.position[1].to_bits(),
            self.resolution[0],
            self.resolution[1],
            self.aspect.to_bits(),
            //Padding cause it wasn't working w/o it
            0,
            self.zoom.to_bits(),
            self.arr_len,
            self.fractal,
            self.max_iter,
            self.num_colors,
            self.msaa,
        ]
    }
}

pub struct Scene {
    pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub buffer: Buffer,
    pub storage_buffer: Buffer,
}

impl Scene {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let (pipeline, buffer, storage_buffer, bind_group) = build_pipeline(device, texture_format);
        Self {
            pipeline,
            bind_group,
            buffer,
            storage_buffer,
        }
    }

    pub fn clear<'a>(
        &self,
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
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
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..6, 0..1);
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
) -> (RenderPipeline, Buffer, Buffer, BindGroup) {
    //Shaders
    let (vs_module, fs_module) = (
        device.create_shader_module(wgpu::include_wgsl!("shader/vert.wgsl")),
        device.create_shader_module(wgpu::include_wgsl!("shader/frag.wgsl")),
    );

    //Uniform buffer creation
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform"),
        contents: bytemuck::cast_slice(&[ShaderDataUniforms::default()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    //Storage buffer for the color array
    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage"),
        //256 floats = 64 vec4s = 64 colors
        contents: bytemuck::cast_slice(&[0.0; 256]),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: storage_buffer.as_entire_binding(),
            },
        ],
    });

    //Pipeline layout creation
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    (
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            //No need for depth
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        }),
        uniform_buffer,
        storage_buffer,
        bind_group,
    )
}
