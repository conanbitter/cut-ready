#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rfd::{AsyncFileDialog, AsyncMessageDialog, MessageButtons, MessageLevel};
use rust_i18n::set_locale;
use softbuffer::{Context, Surface};
use std::fmt::Write;
use std::time::Instant;
use std::{ffi::OsStr, num::NonZeroU32, path::PathBuf, rc::Rc};
use winit::event::MouseButton;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    platform::windows::IconExtWindows,
    window::{Icon, Window, WindowId},
};

use crate::batchconv::try_batch_convert;
use crate::config::{CONFIG, ConfWindow};
use crate::errors::AppError;
use crate::gui::GuiInteractionResult;
use crate::{
    blueprint::{decode_dxf, load_dxf},
    diereader::{convert_die, load_die},
    gui::{Gui, GuiCommand},
    inker::Inker,
};

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");

mod batchconv;
mod blueprint;
mod camera;
mod config;
mod curves;
mod diereader;
mod errors;
mod geometry;
mod gui;
mod inker;
mod pointerrors;

const SPLASH_DATA: &[u8] = include_bytes!("../assets/splash.dxf");

enum MouseAction {
    None,
    Drag,
    Frame,
}

struct AppState<'a> {
    window: Rc<Window>,
    surface: Surface<OwnedDisplayHandle, Rc<Window>>,
    inker: Inker<'a>,
    gui: Gui,
    mouse_x: f64,
    mouse_y: f64,
    old_x: f64,
    old_y: f64,
    mouse_action: MouseAction,
    event_proxy: EventLoopProxy<UserEvent>,
    current_filename: Option<PathBuf>,
    ctrl_pressed: bool,
}

enum UserEvent {
    FileOpenSelected(Option<PathBuf>),
    FileSaveSelected(Option<PathBuf>),
    ExitRequested,
}

impl<'a> AppState<'a> {
    fn open_file_dialog(&self) {
        let task = AsyncFileDialog::new()
            .set_parent(&self.window)
            .set_title(t!("open.title"))
            .add_filter(t!("open.supported"), &["dxf", "die"])
            .add_filter(t!("open.dxf"), &["dxf"])
            .add_filter(t!("open.die"), &["die"])
            .add_filter(t!("open.all"), &["*"])
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
            .set_title(t!("save.title"))
            .add_filter(t!("save.dxf"), &["dxf"])
            .add_filter(t!("save.all"), &["*"])
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

    fn message_box_alone(title: &str, message: &str, message_type: MessageLevel) {
        let dialog = AsyncMessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_level(message_type)
            .set_buttons(MessageButtons::Ok)
            .show();

        std::thread::spawn(move || {
            futures::executor::block_on(dialog);
        });
    }

    fn terminal_error(&self, title: &str, message: &str) {
        let dialog = AsyncMessageDialog::new()
            .set_parent(&self.window)
            .set_title(title)
            .set_description(message)
            .set_level(MessageLevel::Error)
            .set_buttons(MessageButtons::Ok)
            .show();

        let proxy = self.event_proxy.clone();

        std::thread::spawn(move || {
            futures::executor::block_on(dialog);
            let _ = proxy.send_event(UserEvent::ExitRequested);
        });
    }

    fn open_file(&mut self, path: PathBuf, is_splash: bool) {
        if !is_splash && let Some(filename) = path.file_name() {
            self.window
                .set_title(format!("{} - Cut Ready", filename.display()).as_str());
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
        match readed {
            Ok(blueprint) => {
                if blueprint.errors.path.is_none() {
                    self.gui.hide_errors(true, self.mouse_x as i32, self.mouse_y as i32);
                    self.inker.errors_visibile(false);
                } else {
                    self.gui.show_errors(true, self.mouse_x as i32, self.mouse_y as i32);
                    self.inker.errors_visibile(true);
                }
                self.inker.set_blueprint(blueprint);

                if can_save {
                    self.current_filename = Some(path.clone());
                }

                self.window.request_redraw();
            }
            Err(err) => {
                self.message_box(
                    &t!("openerror.title"),
                    format!("{}:\n{}", t!("openerror.text"), err).as_str(),
                    MessageLevel::Error,
                );
            }
        }
    }

    fn command(&mut self, command: GuiCommand) {
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
                self.inker.zoom(1.0 / 0.9, width as f32 / 2.0, height as f32 / 2.0);
            }
            GuiCommand::ZoomOut => {
                let PhysicalSize { width, height } = self.window.inner_size();
                self.inker.zoom(0.9, width as f32 / 2.0, height as f32 / 2.0);
            }
            GuiCommand::ShowAll => {
                self.inker.show_all();
            }
            GuiCommand::ShowErrors => {
                self.inker.errors_visibile(true);
            }
            GuiCommand::HideErrors => {
                self.inker.errors_visibile(false);
            }
            GuiCommand::NextError => {
                self.inker.next_error();
            }
            GuiCommand::PrevError => {
                self.inker.prev_error();
            }
        }

        if !matches!(command, GuiCommand::None) {
            self.window.request_redraw();
        }
    }

    fn color_hint(&self) {
        let mut buffer = String::new();
        let _ = writeln!(buffer, "{}:\n", t!("colorhint.descr"));

        for layer in self.inker.current_blueprint.layers.iter().rev() {
            let layer_type = match layer.hint_type {
                blueprint::LineType::Main => t!("colorhint.main"),
                blueprint::LineType::Cuts => t!("colorhint.cuts"),
                blueprint::LineType::Marks => t!("colorhint.marks"),
                blueprint::LineType::Other => t!("colorhint.other"),
            };
            let r = ((layer.hint_color >> 16) & 0xFF) as u8;
            let g = ((layer.hint_color >> 8) & 0xFF) as u8;
            let b = (layer.hint_color & 0xFF) as u8;
            let _ = writeln!(
                &mut buffer,
                "{0:08}    #{0:06X}   RGB({2:03}, {3:03}, {4:03})   {1}\n",
                layer.hint_color, layer_type, r, g, b
            );
        }

        self.message_box(&t!("colorhint.title"), &buffer, MessageLevel::Info);
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        //let start_time = Instant::now();

        let data = self.inker.draw(&mut self.gui);

        let mut buffer = self.surface.buffer_mut().map_err(|_| AppError::SurfaceDraw)?;

        for (out_pixel, in_pixel) in buffer.iter_mut().zip(data.as_chunks::<4>().0) {
            let red = in_pixel[0] as u32;
            let green = in_pixel[1] as u32;
            let blue = in_pixel[2] as u32;

            *out_pixel = blue | (green << 8) | (red << 16);
        }

        buffer.present().map_err(|_| AppError::SurfaceDraw)?;

        //let frame_duration = Instant::now().duration_since(start_time);
        //self.window.set_title(&frame_duration.as_micros().to_string());

        Ok(())
    }

    fn process_gui_result(&mut self, result: &GuiInteractionResult) {
        if result.request_invalidate {
            self.inker.invalidate();
        }
        if result.request_redraw {
            self.window.request_redraw()
        }
    }

    fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        let nzwidth = NonZeroU32::new(width).ok_or(AppError::SurfaceResize(width, height))?;
        let nzheight = NonZeroU32::new(height).ok_or(AppError::SurfaceResize(width, height))?;
        self.surface
            .resize(nzwidth, nzheight)
            .map_err(|_| AppError::SurfaceResize(width, height))?;
        self.inker.resize(width, height)?;
        Ok(())
    }
}

struct App<'a> {
    context: Context<OwnedDisplayHandle>,
    state: Option<AppState<'a>>,
    proxy: EventLoopProxy<UserEvent>,
    start_file: Option<PathBuf>,
}

impl<'a> App<'a> {
    fn new_window<'b>(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<AppState<'b>> {
        let mut window_size = CONFIG.window;
        if window_size.width == 0 || window_size.height == 0 {
            if let Some(monitor) = event_loop
                .primary_monitor()
                .or_else(|| event_loop.available_monitors().next())
            {
                window_size.width = (monitor.size().width as f32 * 2.0 / 3.0) as i32;
                window_size.height = (monitor.size().height as f32 * 2.0 / 3.0) as i32;
            } else {
                window_size = ConfWindow {
                    width: 1280,
                    height: 720,
                };
            }
        }

        let attributes = Window::default_attributes()
            .with_title("Cut Ready")
            .with_window_icon(Icon::from_resource(1, None).ok())
            .with_inner_size(PhysicalSize::new(window_size.width, window_size.height))
            .with_visible(false);

        let window = event_loop.create_window(attributes)?;

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

        let mut surface = Surface::new(&self.context, window_rc.clone()).map_err(|_| AppError::SurfaceNew)?;
        if let (Some(width), Some(height)) = (NonZeroU32::new(window_size.width), NonZeroU32::new(window_size.height)) {
            surface
                .resize(width, height)
                .map_err(|_| AppError::SurfaceResize(window_size.width, window_size.height))?;
        }

        let gui = Gui::new(2, 2)?;
        let top_height = (2 + gui.height) as f32;

        let mut inker = Inker::new()?;
        let PhysicalSize {
            width: inner_width,
            height: inner_height,
        } = window_rc.inner_size();
        inker.set_viewport(inner_width as f32, inner_height as f32, top_height);
        inker.show_all();

        Ok(AppState {
            window: window_rc,
            surface,
            inker,
            mouse_x: 0.0,
            mouse_y: 0.0,
            old_x: 0.0,
            old_y: 0.0,
            mouse_action: MouseAction::None,
            gui,
            event_proxy: self.proxy.clone(),
            current_filename: None,
            ctrl_pressed: false,
        })
    }
}

impl<'a> ApplicationHandler<UserEvent> for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            match self.new_window(event_loop) {
                Ok(state) => self.state = Some(state),
                Err(err) => AppState::message_box_alone(
                    &t!("wnderror.title"),
                    format!("{}:\n{}", t!("wnderror.text"), err).as_str(),
                    MessageLevel::Error,
                ),
            }

            if let Some(state) = self.state.as_mut() {
                if let Some(start_file) = self.start_file.take() {
                    state.open_file(start_file, false);
                } else {
                    state.open_file(PathBuf::default(), true);
                }
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
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
                        Ok(_) => {
                            if let Some(filename) = die_filename.file_name() {
                                app_state.window.set_title(
                                    format!("{} ({}) - Cut Ready", filename.display(), t!("convert.title_mod"))
                                        .as_str(),
                                );
                            }
                        }
                        Err(err) => app_state.message_box(
                            &t!("convert.error_title"),
                            format!("{}:\n{}", t!("convert.error_text"), err).as_str(),
                            MessageLevel::Error,
                        ),
                    }
                }
            }
            UserEvent::ExitRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(app_state) = self.state.as_mut()
            && let Some(timer) = app_state.gui.timer
        {
            if Instant::now() >= timer {
                let gui_result = app_state.gui.on_timer();
                app_state.process_gui_result(&gui_result);

                app_state.gui.reset_timer();

                event_loop.set_control_flow(ControlFlow::Wait);
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(timer));
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
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
                if let Err(err) = app_state.resize(size.width, size.height) {
                    app_state.terminal_error(
                        &t!("apperror.title"),
                        format!("{}:\n{}", t!("apperror.text"), err).as_str(),
                    )
                }
            }

            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match state {
                winit::event::ElementState::Pressed => {
                    let gui_result = app_state
                        .gui
                        .mouse_press(app_state.mouse_x as i32, app_state.mouse_y as i32);
                    app_state.process_gui_result(&gui_result);

                    if !gui_result.captured {
                        app_state.old_x = app_state.mouse_x;
                        app_state.old_y = app_state.mouse_y;
                        let dirty = match button {
                            MouseButton::Left => {
                                app_state.mouse_action = MouseAction::Drag;
                                app_state.inker.hide_frame();
                                true
                            }
                            MouseButton::Right => {
                                app_state.mouse_action = MouseAction::Frame;
                                app_state.inker.show_frame();
                                app_state.inker.update_frame(
                                    app_state.old_x as f32,
                                    app_state.old_y as f32,
                                    app_state.mouse_x as f32,
                                    app_state.mouse_y as f32,
                                );
                                true
                            }
                            _ => false,
                        };
                        if dirty {
                            app_state.window.request_redraw();
                        }
                    }
                }
                winit::event::ElementState::Released => {
                    let (gui_result, gui_command) = app_state
                        .gui
                        .mouse_release(app_state.mouse_x as i32, app_state.mouse_y as i32);
                    app_state.process_gui_result(&gui_result);
                    if matches!(app_state.mouse_action, MouseAction::Frame) {
                        app_state.inker.zoom_frame(
                            app_state.old_x as f32,
                            app_state.old_y as f32,
                            app_state.mouse_x as f32,
                            app_state.mouse_y as f32,
                        );
                        app_state.inker.hide_frame();
                        app_state.inker.invalidate();
                        app_state.window.request_redraw();
                    }
                    app_state.mouse_action = MouseAction::None;

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
                app_state.process_gui_result(&gui_result);

                match app_state.mouse_action {
                    MouseAction::Drag => {
                        if gui_result.captured {
                            app_state.mouse_action = MouseAction::None;
                        } else {
                            let dx = position.x - app_state.old_x;
                            let dy = position.y - app_state.old_y;
                            app_state.inker.pan(dx as f32, dy as f32);
                            app_state.old_x = position.x;
                            app_state.old_y = position.y;
                            app_state.window.request_redraw();
                        }
                    }
                    MouseAction::Frame if !gui_result.captured => {
                        app_state.inker.update_frame(
                            app_state.old_x as f32,
                            app_state.old_y as f32,
                            app_state.mouse_x as f32,
                            app_state.mouse_y as f32,
                        );
                        app_state.window.request_redraw();
                    }
                    _ => (),
                }
            }

            WindowEvent::CursorLeft { device_id: _ } => {
                if app_state.gui.mouse_leaved() {
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
                        .zoom(factor, app_state.mouse_x as f32, app_state.mouse_y as f32);
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
                    && let PhysicalKey::Code(code) = event.physical_key
                {
                    if !event.repeat {
                        match code {
                            KeyCode::ControlLeft | KeyCode::ControlRight => app_state.ctrl_pressed = true,
                            KeyCode::ArrowRight => {
                                app_state.command(GuiCommand::NextError);
                            }
                            KeyCode::ArrowLeft => {
                                app_state.command(GuiCommand::PrevError);
                            }
                            KeyCode::KeyL if app_state.ctrl_pressed => {
                                app_state.color_hint();
                            }
                            KeyCode::KeyA if app_state.ctrl_pressed => {
                                app_state.command(GuiCommand::ShowAll);
                            }
                            KeyCode::KeyO if app_state.ctrl_pressed => {
                                app_state.command(GuiCommand::Open);
                            }
                            KeyCode::KeyS if app_state.ctrl_pressed && app_state.current_filename.is_some() => {
                                app_state.command(GuiCommand::Save);
                            }
                            KeyCode::KeyE if app_state.ctrl_pressed => {
                                let show_errors = app_state
                                    .gui
                                    .toggle_errors(app_state.mouse_x as i32, app_state.mouse_y as i32);
                                app_state.inker.errors_visibile(show_errors);
                                app_state.window.request_redraw();
                            }
                            _ => (),
                        }
                    }
                    match code {
                        KeyCode::Equal => {
                            app_state.command(GuiCommand::ZoomIn);
                        }
                        KeyCode::Minus => {
                            app_state.command(GuiCommand::ZoomOut);
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
                if let Err(err) = app_state.draw() {
                    app_state.terminal_error(
                        &t!("apperror.title"),
                        format!("{}:\n{}", t!("apperror.text"), err).as_str(),
                    )
                }
            }
            _ => (),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let lang = CONFIG
        .language
        .to_owned()
        .or(sys_locale::get_locale())
        .unwrap_or("en".into());
    set_locale(&lang);

    let Some(start_file) = try_batch_convert() else {
        return Ok(());
    };
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let Ok(context) = Context::new(event_loop.owned_display_handle()) else {
        AppState::message_box_alone(&t!("ctxerror.title"), &t!("ctxerror.text"), MessageLevel::Error);
        return Ok(());
    };
    let proxy = event_loop.create_proxy();

    let mut app = App {
        context,
        state: None,
        proxy,
        start_file,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
