use std::path::PathBuf;

use directories::ProjectDirs;
use display_info::DisplayInfo;

pub fn config_dir() -> PathBuf {
    ProjectDirs::from("io", "luftaquila", "transistor")
        .unwrap()
        .data_local_dir()
        .to_path_buf()
}

pub fn print_displays() {
    println!("[INF] detected system displays:");
    let displays = DisplayInfo::all().unwrap();

    for display in displays {
        println!("  {:?}", display);
    }
}

#[macro_export]
macro_rules! add_warpzone {
    ($disp:expr, $disp_ref:expr, $target:expr) => {
        if let Some((start, end, direction)) = $disp_ref.is_touch($target) {
            $disp_ref.warpzones.push(WarpZone {
                start,
                end,
                direction,
                to: Rc::downgrade($target),
            });

            $target.borrow_mut().warpzones.push(WarpZone {
                start,
                end,
                direction: direction.reverse(),
                to: Rc::downgrade($disp),
            });
        }
    };
}

#[macro_export]
macro_rules! client_point {
    ($x: expr, $y: expr, $target: expr) => {{
        let tgt = $target.upgrade().unwrap();
        let tgt = tgt.borrow();

        WarpPoint {
            x: $x - tgt.x,
            y: $y - tgt.y,
        }
    }};
}

#[macro_export]
macro_rules! tcp_stream_write {
    ($stream:expr, $data:expr) => {
        let encoded = bincode::serialize(&$data).unwrap();

        /* force 4 byte data length */
        let len = encoded.len() as u32;
        let size = len.to_be_bytes();

        if let Err(e) = $stream.write_all(&size) {
            eprintln!("[ERR] TCP stream write failed: {}", e);
        }

        if let Err(e) = $stream.write_all(&encoded) {
            eprintln!("[ERR] TCP stream write failed: {}", e);
        }
    };
}

#[macro_export]
macro_rules! tcp_stream_read {
    ($stream:expr, $buffer:expr) => {
        let mut size = [0u8; 4];

        if let Err(e) = $stream.read_exact(&mut size) {
            return Err(e.into());
        }

        let len = u32::from_be_bytes(size) as usize;

        if let Err(e) = $stream.read_exact(&mut $buffer[..len]) {
            return Err(e.into());
        }
    };
}

#[macro_export]
macro_rules! create_warpgate {
    () => {{
        let event_loop = EventLoop::new();
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

        (window, event_loop)
    }};
}

#[macro_export]
macro_rules! warp {
    ($window:expr, $event_loop:expr) => {
        $window.request_redraw();

        println!("EventLoop started!");

        $event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            // println!("winit: {:?}", event);

            $window
                .set_cursor_position(PhysicalPosition::new(50, 50))
                .unwrap();

            // *control_flow = ControlFlow::Exit;
        });
    };
}
