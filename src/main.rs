use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowBuilderExtWebSys,
    window::WindowBuilder,
};

#[wasm_bindgen(start)]
pub fn start() {
    // Set up panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");

    spawn_local(run());
}

async fn run() {
    // Create the event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_canvas(Some(
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap(),
        ))
        .build(&event_loop)
        .unwrap();

    // Initialize wgpu
    let backend = wgpu::Backends::GL; // Use WebGL backend for broader browser support
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: backend,
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let size = window.inner_size();
    let surface_format = surface.get_capabilities(&adapter).formats[0];
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    // Start the event loop
    event_loop
        .run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            width: new_size.width,
                            height: new_size.height,
                            ..config
                        },
                    );
                }
                Event::RedrawRequested(_) => {
                    let frame = match surface.get_current_texture() {
                        Ok(frame) => frame,
                        Err(_) => {
                            surface.configure(&device, &config);
                            surface
                                .get_current_texture()
                                .expect("Failed to acquire next swap chain texture!")
                        }
                    };
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                    {
                        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0,
                                    }),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });
                        // Rendering commands would go here
                    }

                    queue.submit(std::iter::once(encoder.finish()));
                    frame.present();
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        })
        .unwrap();
}
