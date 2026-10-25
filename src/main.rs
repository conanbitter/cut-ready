use softbuffer::{Context, Surface};
use std::{num::NonZeroU32, rc::Rc};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{blueprint::load_dxf, inker::Inker};

mod blueprint;
mod camera;
mod curves;
mod geometry;
mod inker;
mod pointerrors;

const SPLASH_FILENAME: &str = "tests/test.dxf"; //"splash.dxf";

struct AppState<'a> {
    window: Rc<Window>,
    surface: Surface<OwnedDisplayHandle, Rc<Window>>,
    inker: Inker<'a>,
    mouse_x: f64,
    mouse_y: f64,
    old_x: f64,
    old_y: f64,
    drag: bool,
}

struct App<'a> {
    context: Context<OwnedDisplayHandle>,
    state: Option<AppState<'a>>,
}

impl<'a> ApplicationHandler for App<'a> {
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
            let window_rc = Rc::new(window);

            let mut surface = Surface::new(&self.context, window_rc.clone()).expect("failed creating surface");
            if let (Some(width), Some(height)) =
                (NonZeroU32::new(window_size.width), NonZeroU32::new(window_size.height))
            {
                surface.resize(width, height).unwrap();
            }

            let args: Vec<String> = std::env::args().collect();
            let filename = if args.len() > 1 {
                args[1].clone()
            } else {
                String::from(SPLASH_FILENAME)
            };

            let mut inker = Inker::new(load_dxf(filename).unwrap());
            let PhysicalSize {
                width: inner_width,
                height: inner_height,
            } = window_rc.inner_size();
            inker.camera.set_viewport(inner_width, inner_height);
            inker.show_all();

            self.state = Some(AppState {
                window: window_rc,
                surface,
                inker,
                mouse_x: 0.0,
                mouse_y: 0.0,
                old_x: 0.0,
                old_y: 0.0,
                drag: false,
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(app_state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                    app_state.surface.resize(width, height).unwrap();
                    app_state.inker.resize(size.width, size.height);
                }
            }

            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } => match state {
                winit::event::ElementState::Pressed => {
                    app_state.drag = true;
                    app_state.old_x = app_state.mouse_x;
                    app_state.old_y = app_state.mouse_y;
                }
                winit::event::ElementState::Released => app_state.drag = false,
            },

            WindowEvent::CursorMoved { device_id: _, position } => {
                app_state.mouse_x = position.x;
                app_state.mouse_y = position.y;
                if app_state.drag {
                    let dx = position.x - app_state.old_x;
                    let dy = position.y - app_state.old_y;
                    app_state.inker.camera.pan(dx as f32, dy as f32);
                    app_state.old_x = position.x;
                    app_state.old_y = position.y;
                    app_state.window.request_redraw();
                }
            }

            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    let factor = if y >= 0.0 { 1.0 / 0.9 } else { 0.9 };
                    app_state
                        .inker
                        .camera
                        .scale(factor, app_state.mouse_x as f32, app_state.mouse_y as f32);
                    app_state.window.request_redraw();
                }
                winit::event::MouseScrollDelta::PixelDelta(_) => (),
            },

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.state.is_pressed()
                    && !event.repeat
                    && let PhysicalKey::Code(code) = event.physical_key
                {
                    match code {
                        KeyCode::ArrowRight => {
                            let point = app_state.inker.current_blueprint.errors.next_error();
                            app_state.inker.camera.move_to(point);
                            app_state.window.request_redraw();
                        }
                        KeyCode::ArrowLeft => {
                            let point = app_state.inker.current_blueprint.errors.prev_error();
                            app_state.inker.camera.move_to(point);
                            app_state.window.request_redraw();
                        }
                        _ => (),
                    }
                }
            }

            WindowEvent::DroppedFile(path) => {
                let blueprint = load_dxf(path).unwrap();
                app_state.inker.current_blueprint = blueprint;
                app_state.inker.show_all();
                app_state.window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                let data = app_state.inker.draw();

                let mut buffer = app_state.surface.buffer_mut().unwrap();

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
