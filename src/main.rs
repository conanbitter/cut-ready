#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rfd::{AsyncFileDialog, AsyncMessageDialog, MessageButtons, MessageLevel};
use softbuffer::{Context, Surface};
use std::fmt::Write;
use std::{ffi::OsStr, num::NonZeroU32, path::PathBuf, rc::Rc};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowId},
};

use crate::{
    blueprint::{decode_dxf, load_dxf},
    diereader::{convert_die, load_die},
    gui::{Gui, GuiCommand},
    inker::Inker,
};

mod blueprint;
mod camera;
mod curves;
mod diereader;
mod geometry;
mod gui;
mod inker;
mod pointerrors;

const SPLASH_DATA: &[u8] = include_bytes!("../assets/splash.dxf");

struct AppState<'a> {
    window: Rc<Window>,
    surface: Surface<OwnedDisplayHandle, Rc<Window>>,
    inker: Inker<'a>,
    gui: Gui,
    mouse_x: f64,
    mouse_y: f64,
    old_x: f64,
    old_y: f64,
    drag: bool,
    show_errors: bool,
    event_proxy: EventLoopProxy<UserEvent>,
    current_filename: Option<PathBuf>,
    ctrl_pressed: bool,
}

enum UserEvent {
    FileOpenSelected(Option<PathBuf>),
    FileSaveSelected(Option<PathBuf>),
}

impl<'a> AppState<'a> {
    fn open_file_dialog(&self) {
        let task = AsyncFileDialog::new()
            .set_parent(&self.window)
            .set_title("Open DXF file")
            .add_filter("Supported files", &["dxf", "die"])
            .add_filter("DXF files", &["dxf"])
            .add_filter("DIE files", &["die"])
            .add_filter("All files", &["*"])
            .pick_file();

        let proxy = self.event_proxy.clone();

        std::thread::spawn(move || {
            let path = futures::executor::block_on(task).map(|file| file.path().to_owned());
            let _ = proxy.send_event(UserEvent::FileOpenSelected(path));
        });
    }

    fn save_file_dialog(&self, filename: String) {
        let task = AsyncFileDialog::new()
            .set_parent(&self.window)
            .set_title("Save DXF file")
            .add_filter("DXF files", &["dxf"])
            .add_filter("All files", &["*"])
            .set_file_name(filename)
            .save_file();

        let proxy = self.event_proxy.clone();

        std::thread::spawn(move || {
            let path = futures::executor::block_on(task).map(|file| file.path().to_owned());
            let _ = proxy.send_event(UserEvent::FileSaveSelected(path));
        });
    }

    fn message_box(&self, title: &str, message: &str, message_type: MessageLevel) {
        let dialog = AsyncMessageDialog::new()
            .set_parent(&self.window)
            .set_title(title)
            .set_description(message)
            .set_level(message_type)
            .set_buttons(MessageButtons::Ok)
            .show();

        std::thread::spawn(move || {
            futures::executor::block_on(dialog);
        });
    }

    fn open_file(&mut self, path: PathBuf, is_splash: bool) {
        if !is_splash {
            self.window
                .set_title(format!("{} - Cut Ready", path.file_name().unwrap().display()).as_str());
        }

        let mut can_save = false;

        let readed = if is_splash {
            decode_dxf(SPLASH_DATA)
        } else if path.extension() == Some(OsStr::new("die")) {
            if !is_splash {
                self.gui.enable_saving();
                can_save = true;
            }
            load_die(path.clone())
        } else {
            self.gui.disable_saving();
            load_dxf(path.clone())
        };
        //println!("{}", path.extension().unwrap().display());
        match readed {
            Ok(blueprint) => {
                if blueprint.errors.path.is_none() {
                    self.gui.hide_errors(true, self.mouse_x as i32, self.mouse_y as i32);
                    self.show_errors = false;
                } else {
                    self.gui.show_errors(true, self.mouse_x as i32, self.mouse_y as i32);
                    self.show_errors = true;
                }
                self.inker.current_blueprint = blueprint;
                self.inker.show_all();

                if can_save {
                    self.current_filename = Some(path.clone());
                }

                self.window.request_redraw();
            }
            Err(err) => {
                self.message_box(
                    "File error",
                    format!("Error opening file\n{}", err).as_str(),
                    MessageLevel::Error,
                );
            }
        }
    }

    fn command(&mut self, command: GuiCommand) {
        //println!("Command {:?}", command);
        match command {
            GuiCommand::None => (),
            GuiCommand::Open => {
                self.open_file_dialog();
            }
            GuiCommand::Save => {
                if let Some(filename) = self.current_filename.as_ref() {
                    let new_filename = match filename.with_extension("dxf").file_name() {
                        Some(name) => name.to_string_lossy().into_owned(),
                        None => String::default(),
                    };
                    self.save_file_dialog(new_filename);
                }
            }
            GuiCommand::ZoomIn => {
                let PhysicalSize { width, height } = self.window.inner_size();
                self.inker
                    .camera
                    .scale(1.0 / 0.9, width as f32 / 2.0, height as f32 / 2.0);
            }
            GuiCommand::ZoomOut => {
                let PhysicalSize { width, height } = self.window.inner_size();
                self.inker.camera.scale(0.9, width as f32 / 2.0, height as f32 / 2.0);
            }
            GuiCommand::ShowAll => {
                self.inker.show_all();
            }
            GuiCommand::ShowErrors => {
                self.show_errors = true;
            }
            GuiCommand::HideErrors => {
                self.show_errors = false;
            }
            GuiCommand::NextError => {
                let point = self.inker.current_blueprint.errors.next_error();
                self.inker.camera.move_to(point);
            }
            GuiCommand::PrevError => {
                let point = self.inker.current_blueprint.errors.prev_error();
                self.inker.camera.move_to(point);
            }
        }
    }

    fn color_hint(&self) {
        let mut buffer = String::from("Line colors:\n\n");

        for layer in &self.inker.current_blueprint.layers {
            let layer_type = match layer.hint_type {
                blueprint::LineType::Main => "main",
                blueprint::LineType::Cuts => "cuts",
                blueprint::LineType::Marks => "marks",
                blueprint::LineType::Other => "other",
            };
            let _ = writeln!(&mut buffer, "{0:06X}   {0:08}   {1}", layer.hint_color, layer_type);
        }

        self.message_box("Colors", &buffer, MessageLevel::Info);
    }
}

struct App<'a> {
    context: Context<OwnedDisplayHandle>,
    state: Option<AppState<'a>>,
    proxy: EventLoopProxy<UserEvent>,
}

impl<'a> ApplicationHandler<UserEvent> for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Cut Ready")
                .with_window_icon(Icon::from_resource(1, None).ok())
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

            let gui = Gui::new(2, 2);
            let top_height = (2 + gui.height) as f32;

            let mut inker = Inker::new();
            let PhysicalSize {
                width: inner_width,
                height: inner_height,
            } = window_rc.inner_size();
            inker
                .camera
                .set_viewport(inner_width as f32, inner_height as f32, top_height);
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
                gui,
                show_errors: false,
                event_proxy: self.proxy.clone(),
                current_filename: None,
                ctrl_pressed: false,
            });

            if let Some(state) = self.state.as_mut() {
                let args: Vec<String> = std::env::args().collect();
                if args.len() > 1 {
                    state.open_file(PathBuf::from(args[1].clone()), false);
                } else {
                    state.open_file(PathBuf::default(), true);
                };
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        let Some(app_state) = self.state.as_mut() else {
            return;
        };

        match event {
            UserEvent::FileOpenSelected(Some(path)) => {
                app_state.open_file(path, false);
            }
            UserEvent::FileSaveSelected(Some(path)) => {
                if let Some(die_filename) = app_state.current_filename.as_ref() {
                    match convert_die(die_filename, path) {
                        Ok(_) => app_state.message_box("Success", "File successfully converted!", MessageLevel::Info),
                        Err(err) => app_state.message_box(
                            "Convert error",
                            format!("Error converting file\n{}", err).as_str(),
                            MessageLevel::Error,
                        ),
                    }
                }
            }
            _ => (),
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
                    let gui_result = app_state
                        .gui
                        .mouse_press(app_state.mouse_x as i32, app_state.mouse_y as i32);
                    if !gui_result.captured {
                        app_state.drag = true;
                        app_state.old_x = app_state.mouse_x;
                        app_state.old_y = app_state.mouse_y;
                    }
                    if gui_result.needs_redraw {
                        app_state.window.request_redraw();
                    }
                }
                winit::event::ElementState::Released => {
                    let (gui_result, gui_command) = app_state
                        .gui
                        .mouse_release(app_state.mouse_x as i32, app_state.mouse_y as i32);
                    app_state.drag = false;
                    if gui_result.needs_redraw {
                        app_state.window.request_redraw();
                    } else {
                        match gui_command {
                            GuiCommand::None => (),
                            _ => app_state.window.request_redraw(),
                        }
                    }
                    match gui_command {
                        GuiCommand::None => (),
                        _ => app_state.command(gui_command),
                    }
                }
            },

            WindowEvent::CursorMoved { device_id: _, position } => {
                app_state.mouse_x = position.x;
                app_state.mouse_y = position.y;
                let gui_result = app_state.gui.mouse_move(position.x as i32, position.y as i32);
                let mut needs_redraw = gui_result.needs_redraw;
                if gui_result.captured {
                    app_state.drag = false;
                } else {
                    if app_state.drag {
                        let dx = position.x - app_state.old_x;
                        let dy = position.y - app_state.old_y;
                        app_state.inker.camera.pan(dx as f32, dy as f32);
                        app_state.old_x = position.x;
                        app_state.old_y = position.y;
                        needs_redraw = true;
                    }
                }
                if needs_redraw {
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
                        KeyCode::ControlLeft | KeyCode::ControlRight => app_state.ctrl_pressed = true,
                        KeyCode::ArrowRight => {
                            if app_state.show_errors {
                                app_state.command(GuiCommand::NextError);
                                app_state.window.request_redraw();
                            }
                        }
                        KeyCode::ArrowLeft => {
                            if app_state.show_errors {
                                app_state.command(GuiCommand::PrevError);
                                app_state.window.request_redraw();
                            }
                        }
                        KeyCode::Equal => {
                            app_state.command(GuiCommand::ZoomIn);
                            app_state.window.request_redraw();
                        }
                        KeyCode::Minus => {
                            app_state.command(GuiCommand::ZoomOut);
                            app_state.window.request_redraw();
                        }
                        KeyCode::KeyC if app_state.ctrl_pressed => {
                            app_state.color_hint();
                        }
                        KeyCode::KeyA if app_state.ctrl_pressed => {
                            app_state.command(GuiCommand::ShowAll);
                            app_state.window.request_redraw();
                        }
                        KeyCode::KeyO if app_state.ctrl_pressed => {
                            app_state.command(GuiCommand::Open);
                        }
                        KeyCode::KeyS if app_state.ctrl_pressed && app_state.current_filename.is_some() => {
                            app_state.command(GuiCommand::Save);
                        }
                        KeyCode::KeyE if app_state.ctrl_pressed => {
                            app_state.show_errors = app_state
                                .gui
                                .toggle_errors(app_state.mouse_x as i32, app_state.mouse_y as i32);
                            app_state.window.request_redraw();
                        }
                        _ => (),
                    }
                } else if !event.state.is_pressed()
                    && let PhysicalKey::Code(code) = event.physical_key
                {
                    match code {
                        KeyCode::ControlLeft | KeyCode::ControlRight => app_state.ctrl_pressed = false,
                        _ => (),
                    }
                }
            }

            WindowEvent::DroppedFile(path) => {
                app_state.open_file(path, false);
            }

            WindowEvent::RedrawRequested => {
                let data = app_state.inker.draw(&app_state.gui, app_state.show_errors);

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

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let context = Context::new(event_loop.owned_display_handle()).unwrap();
    let proxy = event_loop.create_proxy();

    let mut app = App {
        context,
        state: None,
        proxy,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
