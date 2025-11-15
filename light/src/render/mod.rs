use glam::{Vec3, vec2, vec3};
use pollster::FutureExt;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, Device, DeviceDescriptor, FragmentState, Instance,
    InstanceDescriptor, LoadOp, Operations, PipelineLayoutDescriptor, PowerPreference,
    PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, SurfaceTargetUnsafe,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use wgpu::{AdapterInfo, CommandEncoderDescriptor, TextureViewDescriptor};
use winit::{dpi::PhysicalSize, window::Window};

use crate::asset::{Mesh, Vertex};
use crate::camera::Camera;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    forward: Vec3,
    fov: f32,
    position: Vec3,
    aspect_ratio: f32,
}

pub struct Renderer {
    surface: Surface<'static>,
    adapter: Adapter,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,

    render_pipeline: RenderPipeline,
    fullscreen_triangle: MeshBuffer,
    bind_group_layout: BindGroupLayout,
    uniform_buffer: Buffer,

    window: Window,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let instance = Instance::new(&InstanceDescriptor::default());

        // SAFETY: Window has the same lifetime as surface
        let surface = unsafe {
            instance
                .create_surface_unsafe(SurfaceTargetUnsafe::from_window(&window).unwrap())
                .unwrap()
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .unwrap();

        let inner_size = window.inner_size();
        let surface_config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .block_on()
            .unwrap();

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[vertex_layout()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let mut mesh = Mesh::new();
        mesh.add_vertex(Vertex {
            position: vec3(-1.0, 3.0, 0.0),
            normal: vec3(0.0, 0.0, 1.0),
            texcoord: vec2(0.0, 4.0),
        });
        mesh.add_vertex(Vertex {
            position: vec3(-1.0, -1.0, 0.0),
            normal: vec3(0.0, 0.0, 1.0),
            texcoord: vec2(0.0, 0.0),
        });
        mesh.add_vertex(Vertex {
            position: vec3(3.0, -1.0, 0.0),
            normal: vec3(0.0, 0.0, 1.0),
            texcoord: vec2(4.0, 0.0),
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mesh.vertex_data()),
            usage: BufferUsages::VERTEX,
        });

        let fullscreen_triangle = MeshBuffer {
            vertex_buffer,
            index_buffer: None,
            num_indices: 0,
            num_vertices: mesh.num_vertices(),
        };

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<ShaderUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            surface,
            adapter,
            surface_config,
            device,
            queue,

            render_pipeline,
            fullscreen_triangle,
            bind_group_layout,
            uniform_buffer,

            window,
        };

        renderer.resize(inner_size);

        renderer
    }

    pub fn create_mesh_buffer(&self, mesh: &Mesh) -> MeshBuffer {
        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mesh.vertex_data()),
            usage: BufferUsages::VERTEX,
        });

        MeshBuffer {
            vertex_buffer,
            index_buffer: None,
            num_indices: 0,
            num_vertices: mesh.num_vertices(),
        }
    }

    pub fn create_data_buffer(&self, data: &[u8]) -> DataBuffer {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &data,
            usage: BufferUsages::STORAGE,
        });

        DataBuffer { buffer }
    }

    pub fn adapter_info(&self) -> AdapterInfo {
        self.adapter.get_info()
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface_config.width = size.width;
        self.surface_config.height = size.height;

        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render(&mut self, camera: &Camera, data: &DataBuffer) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let (forward, _) = camera.forward_right();
        let fov = camera.fov.to_radians();

        let inner_size = self.window.inner_size();
        let aspect_ratio = inner_size.width as f32 / inner_size.height as f32;

        let uniforms = ShaderUniforms {
            forward,
            fov,
            position: camera.position,
            aspect_ratio,
        };

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: data.buffer.as_entire_binding(),
                },
            ],
        });

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.fullscreen_triangle.vertex_buffer.slice(..));
            render_pass.draw(0..self.fullscreen_triangle.num_vertices, 0..1);
        }

        self.queue.submit([encoder.finish()]);

        surface_texture.present();
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

pub struct MeshBuffer {
    vertex_buffer: Buffer,
    index_buffer: Option<Buffer>,
    num_indices: u32,
    num_vertices: u32,
}

const ATTRIBUTES: [VertexAttribute; 3] = [
    VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: VertexFormat::Float32x3,
    },
    VertexAttribute {
        offset: 3 * 4,
        shader_location: 1,
        format: VertexFormat::Float32x3,
    },
    VertexAttribute {
        offset: 6 * 4,
        shader_location: 2,
        format: VertexFormat::Float32x2,
    },
];

fn vertex_layout() -> VertexBufferLayout<'static> {
    VertexBufferLayout {
        array_stride: 8 * 4,
        step_mode: VertexStepMode::Vertex,
        attributes: &ATTRIBUTES,
    }
}

pub struct DataBuffer {
    buffer: Buffer,
}
