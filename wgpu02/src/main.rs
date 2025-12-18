use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

fn main() {
    let window_attributes = Window::default_attributes();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let window = event_loop.create_window(window_attributes).unwrap();
    window.set_title("My window");
    //env_logger::init();

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => { elwt.exit(); },
            _ => {}
        }
    });
}
