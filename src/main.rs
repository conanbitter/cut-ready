//use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap())
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
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

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
