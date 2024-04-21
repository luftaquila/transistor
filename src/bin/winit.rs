use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use pixels::{Pixels, SurfaceTexture};

fn main() {
    let size_i = PhysicalSize::new(1, 1);
    let size = PhysicalSize::new(100, 100);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false)
        .with_inner_size(size_i)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();
    window.set_cursor_visible(false);

    let mut pixels = Pixels::new(
        size_i.width,
        size_i.height,
        SurfaceTexture::new(size_i.width, size_i.height, &window),
    )
    .unwrap();

    for pixel in pixels.frame_mut().chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x80]);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::RedrawRequested(_) = event {
            pixels.render().unwrap();
        }

        window.set_inner_size(size);
        window.set_visible(true);

        window.request_redraw();
    });
}
