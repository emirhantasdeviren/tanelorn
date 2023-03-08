use render::Renderer;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::init();
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial")
        .with_inner_size(PhysicalSize::new(800, 600))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let _renderer = Renderer::new(&window);

    event_loop.run_return(|event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.set_exit(),
                _ => (),
            },
            _ => (),
        }
    });
}
