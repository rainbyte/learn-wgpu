use std::{iter, mem};
use std::sync::Arc;

use bytemuck::{ Pod, Zeroable, cast_slice };
use cgmath::{ Matrix, Matrix4, SquareMatrix };
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window
};

#[path="../src/transforms.rs"]
mod transforms;

const IS_PERSPECTIVE : bool = true;
const ANIMATION_SPEED : f32 = 1.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Light {
    color: [f32; 4],
    specular_color: [f32; 4],
    ambient_intensity: f32,
    diffuse_intensity: f32,
    specular_intensity: f32,
    specular_shininess: f32,
}

pub fn light(c: [f32; 3], sc: [f32; 3], ai: f32, di: f32, si: f32, ss: f32) -> Light {
    Light {
        color: [c[0], c[1], c[2], 1.0],
        specular_color: [sc[0], sc[1], sc[2], 1.0],
        ambient_intensity: ai,
        diffuse_intensity: di,
        specular_intensity: si,
        specular_shininess: ss,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct State<'a> {
    init: transforms::InitWgpu<'a>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_uniform_buffer: wgpu::Buffer,
    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    num_vertices: u32,
    window: Arc<Window>,
}

impl State<'_> {
    pub async fn new(
        window: Arc<Window>, vertex_data: &Vec<Vertex>, light_data: Light
    ) -> Self {
        let init = transforms::InitWgpu::init_wgpu(window.clone()).await;

        // Load the shaders from disk
        let shader = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // uniform data
        let camera_position = (3.0, 1.5, 3.0).into();
        let look_direction = (0.0, 0.0, 0.0).into();
        let up_direction = cgmath::Vector3::unit_y();

        let model_mat = transforms::create_transforms(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0]
        );
        let (view_mat, project_mat, _) =
            transforms::create_view_projection(
                camera_position, look_direction, up_direction,
                init.config.width as f32 / init.config.height as f32,
                IS_PERSPECTIVE
            );

        let vertex_uniform_buffer = init.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Vertex Uniform Buffer"),
                size: 192,
                usage: wgpu::BufferUsages::UNIFORM
                     | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let fragment_uniform_buffer = init.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Fragment Uniform Buffer"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM
                     | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let light_position: &[f32; 3] = camera_position.as_ref();
        let eye_position: &[f32; 3] = camera_position.as_ref();
        init.queue.write_buffer(
            &fragment_uniform_buffer, 0, bytemuck::cast_slice(light_position)
        );
        init.queue.write_buffer(
            &fragment_uniform_buffer, 16, bytemuck::cast_slice(eye_position)
        );

        let light_uniform_buffer = init.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Light Uniform Buffer"),
                size: 48,
                usage: wgpu::BufferUsages::UNIFORM
                     | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        init.queue.write_buffer(
            &light_uniform_buffer, 0, bytemuck::cast_slice(&[light_data])
        );

        let uniform_bind_group_layout = init.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
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
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("Uniform Bind Group Layout"),
            }
        );

        let uniform_bind_group = init.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: vertex_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: fragment_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: light_uniform_buffer.as_entire_binding(),
                    },
                ],
                label: Some("Uniform Bind Group"),
            }
        );

        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: init.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let vertex_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = vertex_data.len() as u32;

        Self {
            init,
            pipeline,
            vertex_buffer,
            uniform_bind_group,
            vertex_uniform_buffer,
            view_mat,
            project_mat,
            num_vertices,
            window,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // Recreate the surface with the new size
            self.init.instance.poll_all(true);
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init.surface.configure(&self.init.device, &self.init.config);

            self.project_mat = transforms::create_projection(
                new_size.width as f32 / new_size.height as f32, IS_PERSPECTIVE
            );
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        // update uniform buffer
        let dt = ANIMATION_SPEED * dt.as_secs_f32();
        let model_mat = transforms::create_transforms(
            [0.0, 0.0, 0.0],
            [dt.sin(), dt.cos(), 0.0],
            [1.0, 1.0, 1.0]
        );
        let view_project_mat = self.project_mat * self.view_mat;

        let normal_mat = (model_mat.invert().unwrap()).transpose();

        let model_ref: &[f32; 16] = model_mat.as_ref();
        let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();
        let normal_ref: &[f32; 16] = normal_mat.as_ref();

        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            0,
            bytemuck::cast_slice(model_ref)
        );
        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            64,
            bytemuck::cast_slice(view_projection_ref)
        );
        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            128,
            bytemuck::cast_slice(normal_ref)
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();
        let frame = self.init.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = self.init.device.create_texture(
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: self.init.config.width,
                    height: self.init.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
                view_formats: &[],
            });
        let depth_view = depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .init
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0
                            }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None
                    }
                ),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
            rpass.draw(0..36, 0..1);
        }
        self.init.queue.submit(iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}

pub fn run(vertex_data: &Vec<Vertex>, light_data: Light, title: &str) {
    let window_attributes = Window::default_attributes();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        event_loop.create_window(window_attributes).unwrap()
    );
    window.set_title(&*format!("{}", title));

    let mut state = pollster::block_on(
        State::new(window, &vertex_data, light_data)
    );

    let start_time = std::time::Instant::now();
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                ..
            } => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = now - start_time;
                        state.update(dt);
                        match state.render() {
                            Ok(_) => {},
                            Err(wgpu::SurfaceError::Lost)
                                => state.resize(state.init.size),
                            Err(wgpu::SurfaceError::OutOfMemory)
                                => elwt.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    },
                    WindowEvent::Resized(size) => state.resize(*size),
                    _ => {}
                }
            },
            _ => {}
        }
    });
}
