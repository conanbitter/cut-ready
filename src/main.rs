//use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use std::num::NonZeroU32;

use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};

struct App {
    context: Context<OwnedDisplayHandle>,
    surface: Option<Surface<OwnedDisplayHandle, Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Cut Ready")
                .with_inner_size(PhysicalSize::new(1280, 720))
                .with_visible(false);

            let window = event_loop.create_window(attributes).unwrap();

            let target_monitor = window.current_monitor().or_else(|| window.available_monitors().next());

            if let Some(monitor) = target_monitor {
                let monitor_size = monitor.size();
                let window_size = window.outer_size();

                let x = (monitor_size.width.saturating_sub(window_size.width) / 2) as i32;
                let y = (monitor_size.height.saturating_sub(window_size.height) / 2) as i32;

                let monitor_pos = monitor.position();
                let centered_pos = PhysicalPosition::new(x + monitor_pos.x, y + monitor_pos.y);

                window.set_outer_position(centered_pos);
            }

            window.set_visible(true);
            let window_size = window.inner_size();

            let mut surface = Surface::new(&self.context, window).expect("failed creating surface");
            if let (Some(width), Some(height)) =
                (NonZeroU32::new(window_size.width), NonZeroU32::new(window_size.height))
            {
                surface.resize(width, height).unwrap();
            }

            self.surface = Some(surface);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                    surface.resize(width, height).unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                let mut buffer = surface.buffer_mut().unwrap();

                // Render into the buffer.
                for (i, pixel) in buffer.iter_mut().enumerate() {
                    let red = (i % 255) as u32;
                    let green = ((i + 50) % 255) as u32;
                    let blue = ((i + 100) % 255) as u32;

                    *pixel = blue | (green << 8) | (red << 16);
                }

                // Send the buffer to the compositor.
                buffer.present().unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    /*let mut pixmap = Pixmap::new(256, 256).unwrap();
    pixmap.fill(Color::WHITE);

    let mut pb = PathBuilder::new();
    pb.move_to(128.0, 32.0);
    pb.line_to(224.0, 224.0);
    pb.line_to(32.0, 224.0);
    pb.close();
    let path = pb.finish().unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 5.0;
    stroke.line_join = tiny_skia::LineJoin::Round;

    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    //pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

    pixmap.save_png("triangle.png").unwrap();*/
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let context = Context::new(event_loop.owned_display_handle()).unwrap();

    let mut app = App { context, surface: None };
    event_loop.run_app(&mut app).unwrap();
}
