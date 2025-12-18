mod common;

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

fn main() {
    let window_attributes = Window::default_attributes();
    let event_loop = EventLoop::new().unwrap();
    let window = event_loop.create_window(window_attributes).unwrap();
    window.set_title(&*format!("{}", "Square"));

    let mut state = pollster::block_on(common::State::new(window));
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
                        state.update();
                        match state.render() {
                            Ok(_) => {},
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
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
