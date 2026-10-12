use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};

use crate::inker::Inker;

mod inker;

struct AppState {
    surface: Surface<OwnedDisplayHandle, Window>,
    inker: Inker,
}

struct App {
    context: Context<OwnedDisplayHandle>,
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
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

            self.state = Some(AppState {
                surface,
                inker: Inker::new(),
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                    state.surface.resize(width, height).unwrap();
                    state.inker.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let data = state.inker.draw();

                let mut buffer = state.surface.buffer_mut().unwrap();

                for (out_pixel, in_pixel) in buffer.iter_mut().zip(data.as_chunks::<4>().0) {
                    let red = in_pixel[0] as u32;
                    let green = in_pixel[1] as u32;
                    let blue = in_pixel[2] as u32;

                    *out_pixel = blue | (green << 8) | (red << 16);
                }

                buffer.present().unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let context = Context::new(event_loop.owned_display_handle()).unwrap();

    let mut app = App { context, state: None };
    event_loop.run_app(&mut app).unwrap();
}
