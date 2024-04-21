use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use pixels::{Pixels, SurfaceTexture};

fn main() {
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false)
        .with_inner_size(PhysicalSize::new(1, 1))
        .with_position(PhysicalPosition::new(0, 0))
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut pixels = Pixels::new(1, 1, SurfaceTexture::new(1, 1, &window)).unwrap();

    for pixel in pixels.frame_mut().chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x80]);
    }

    window.set_inner_size(PhysicalSize::new(100, 100));
    window.set_cursor_visible(false);
    window.set_visible(true);

    pixels.render().unwrap();

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        println!("{:?}", event);

        window
            .set_cursor_position(PhysicalPosition::new(50, 50))
            .unwrap();
        // *control_flow = ControlFlow::Exit;
    });
}
