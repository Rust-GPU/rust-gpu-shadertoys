use futures::executor::block_on;
use ouroboros::self_referencing;
use std::error::Error;
use std::time::Instant;
use wgpu::{self, InstanceDescriptor};
use wgpu::{include_spirv, include_spirv_raw};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::NamedKey;
use winit::window::{Window, WindowAttributes, WindowId};

#[self_referencing]
struct WindowSurface {
    window: Box<dyn Window>,
    #[borrows(window)]
    #[covariant]
    surface: wgpu::Surface<'this>,
}

struct ShaderToyApp {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    window_surface: Option<WindowSurface>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    shader_module: Option<wgpu::ShaderModule>,
    close_requested: bool,
    start: Instant,
    // Mouse state.
    cursor_x: f32,
    cursor_y: f32,
    drag_start_x: f32,
    drag_start_y: f32,
    drag_end_x: f32,
    drag_end_y: f32,
    mouse_left_pressed: bool,
    mouse_left_clicked: bool,
}

impl Default for ShaderToyApp {
    fn default() -> Self {
        Self {
            device: None,
            queue: None,
            window_surface: None,
            config: None,
            render_pipeline: None,
            shader_module: None,
            close_requested: false,
            start: Instant::now(),
            cursor_x: 0.0,
            cursor_y: 0.0,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            drag_end_x: 0.0,
            drag_end_y: 0.0,
            mouse_left_pressed: false,
            mouse_left_clicked: false,
        }
    }
}

impl ShaderToyApp {
    async fn init(&mut self, event_loop: &dyn ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let window_attributes = WindowAttributes::default()
            .with_title("Rust GPU - wgpu")
            .with_surface_size(LogicalSize::new(1280.0, 720.0));
        let window_box = event_loop.create_window(window_attributes)?;
        let mut instance_flags = wgpu::InstanceFlags::default();
        // Turn off validation as the shaders are trusted.
        instance_flags.remove(wgpu::InstanceFlags::VALIDATION);
        // Disable debugging info to speed things up.
        instance_flags.remove(wgpu::InstanceFlags::DEBUG);
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            flags: instance_flags,
            ..Default::default()
        });

        let window_surface = WindowSurfaceBuilder {
            window: window_box,
            surface_builder: |window| {
                instance
                    .create_surface(window)
                    .expect("Failed to create surface")
            },
        }
        .build();

        let window_size = window_surface.borrow_window().surface_size();
        let surface = window_surface.borrow_surface();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await?;
        let mut required_features = wgpu::Features::PUSH_CONSTANTS;
        if adapter
            .features()
            .contains(wgpu::Features::SPIRV_SHADER_PASSTHROUGH)
        {
            required_features |= wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
        }
        let required_limits = wgpu::Limits {
            max_push_constant_size: 256,
            ..Default::default()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                required_limits,
                ..Default::default()
            })
            .await?;
        let shader_module = if device
            .features()
            .contains(wgpu::Features::SPIRV_SHADER_PASSTHROUGH)
        {
            let x = include_spirv_raw!(env!(
                "shadertoys_shaders.spv"
            ));
            unsafe {
                device.create_shader_module_passthrough(x)
            }
        } else {
            device.create_shader_module(include_spirv!(env!("shadertoys_shaders.spv")))
        };
        let swapchain_format = surface.get_capabilities(&adapter).formats[0];
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..std::mem::size_of::<shared::ShaderConstants>() as u32,
            }],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("main_vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("main_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: Default::default(),
        };
        surface.configure(&device, &config);

        self.device = Some(device);
        self.queue = Some(queue);
        self.window_surface = Some(window_surface);
        self.config = Some(config);
        self.render_pipeline = Some(render_pipeline);
        self.shader_module = Some(shader_module);
        self.start = Instant::now();
        Ok(())
    }

    fn render(&mut self) {
        let window_surface = match &self.window_surface {
            Some(ws) => ws,
            None => return,
        };

        let window = window_surface.borrow_window();
        let current_size = window.surface_size();
        let surface = window_surface.borrow_surface();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to acquire texture: {:?}", e);
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_viewport(
                0.0,
                0.0,
                current_size.width as f32,
                current_size.height as f32,
                0.0,
                1.0,
            );
            let push_constants = shared::ShaderConstants {
                width: current_size.width,
                height: current_size.height,
                time: self.start.elapsed().as_secs_f32(),
                cursor_x: self.cursor_x,
                cursor_y: self.cursor_y,
                drag_start_x: self.drag_start_x,
                drag_start_y: self.drag_start_y,
                drag_end_x: self.drag_end_x,
                drag_end_y: self.drag_end_y,
                mouse_left_pressed: self.mouse_left_pressed as u32,
                mouse_left_clicked: self.mouse_left_clicked as u32,
            };
            self.mouse_left_clicked = false;
            rpass.set_pipeline(self.render_pipeline.as_ref().unwrap());
            rpass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::bytes_of(&push_constants),
            );
            rpass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl ApplicationHandler for ShaderToyApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Err(e) = block_on(self.init(event_loop)) {
            eprintln!("Initialization error: {e}");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.close_requested = true,
            WindowEvent::SurfaceResized(new_size) => {
                if let Some(config) = self.config.as_mut() {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    if let Some(ws) = &self.window_surface {
                        let surface = ws.borrow_surface();
                        if let Some(device) = self.device.as_ref() {
                            surface.configure(device, config);
                        }
                    }
                }
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.cursor_x = position.x as f32;
                self.cursor_y = position.y as f32;
                if self.mouse_left_pressed {
                    self.drag_end_x = self.cursor_x;
                    self.drag_end_y = self.cursor_y;
                }
            }
            WindowEvent::PointerButton { state, button, .. } => {
                if button.mouse_button() == MouseButton::Left {
                    self.mouse_left_pressed = state == ElementState::Pressed;
                    if self.mouse_left_pressed {
                        self.drag_start_x = self.cursor_x;
                        self.drag_start_y = self.cursor_y;
                        self.drag_end_x = self.cursor_x;
                        self.drag_end_y = self.cursor_y;
                        self.mouse_left_clicked = true;
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                    self.drag_end_x = x * 0.1;
                    self.drag_end_y = y * 0.1;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == NamedKey::Escape && event.state == ElementState::Pressed {
                    self.close_requested = true;
                }
            }
            WindowEvent::RedrawRequested => self.render(),
            _ => {}
        }
        if self.close_requested {
            event_loop.exit();
        } else if let Some(ws) = &self.window_surface {
            ws.borrow_window().request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.close_requested {
            event_loop.exit();
        } else if let Some(ws) = &self.window_surface {
            ws.borrow_window().request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let mut app = ShaderToyApp::default();
    event_loop.run_app(&mut app).map_err(Into::into)
}
