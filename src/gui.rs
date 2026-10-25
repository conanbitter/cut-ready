use tiny_skia::{BlendMode, FilterQuality, Pixmap, PixmapPaint, Transform};

pub const GUI_PAINT: PixmapPaint = PixmapPaint {
    opacity: 1.0,
    blend_mode: BlendMode::SourceOver,
    quality: FilterQuality::Nearest,
};

pub const GUI_TRANSFORM: Transform = Transform {
    sx: 1.0,
    kx: 0.0,
    ky: 0.0,
    sy: 1.0,
    tx: 0.0,
    ty: 0.0,
};

const BUTTON_SPACING: i32 = 2;
const BUTTON_WIDE_SPACING: i32 = 10;

#[derive(Copy, Clone)]
enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

struct Button {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    state: ButtonState,
    image_normal: Pixmap,
    image_hovered: Pixmap,
    image_pressed: Pixmap,
    image_disabled: Option<Pixmap>,
    visible: bool,
    command: GuiCommand,
}

#[derive(Default)]
pub struct GuiInteractionResult {
    pub captured: bool,
    pub needs_redraw: bool,
}

impl Button {
    fn new(x: i32, y: i32, name: &str, command: GuiCommand, visible: bool) -> Button {
        let image_normal = Pixmap::load_png(format!("WIP/{}_normal.png", name)).unwrap();
        let image_hovered = Pixmap::load_png(format!("WIP/{}_hovered.png", name)).unwrap();
        let image_pressed = Pixmap::load_png(format!("WIP/{}_pressed.png", name)).unwrap();
        let image_disabled = Pixmap::load_png(format!("WIP/{}_disabled.png", name)).ok();
        let width = image_normal.width() as i32;
        let height = image_normal.height() as i32;
        Button {
            x,
            y,
            width,
            height,
            state: ButtonState::Normal,
            image_normal,
            image_hovered,
            image_pressed,
            image_disabled,
            visible,
            command,
        }
    }

    fn draw(&self, pixmap: &mut Pixmap) {
        if !self.visible {
            return;
        }
        let image = match self.state {
            ButtonState::Normal => &self.image_normal,
            ButtonState::Hovered => &self.image_hovered,
            ButtonState::Pressed => &self.image_pressed,
            ButtonState::Disabled => {
                if let Some(image) = self.image_disabled.as_ref() {
                    image
                } else {
                    &self.image_normal
                }
            }
        };
        pixmap.draw_pixmap(self.x, self.y, image.as_ref(), &GUI_PAINT, GUI_TRANSFORM, None);
    }

    fn mouse_inbound(&self, x: i32, y: i32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn mouse_move(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        if !self.visible {
            return GuiInteractionResult {
                captured: false,
                needs_redraw: false,
            };
        }
        let inbound = self.mouse_inbound(x, y);
        let needs_redraw = match self.state {
            ButtonState::Normal => {
                if inbound {
                    self.state = ButtonState::Hovered;
                    true
                } else {
                    false
                }
            }
            ButtonState::Hovered | ButtonState::Pressed => {
                if inbound {
                    false
                } else {
                    self.state = ButtonState::Normal;
                    true
                }
            }
            ButtonState::Disabled => false,
        };
        GuiInteractionResult {
            captured: inbound,
            needs_redraw,
        }
    }

    fn mouse_press(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        if !self.visible {
            return GuiInteractionResult {
                captured: false,
                needs_redraw: false,
            };
        }
        let inbound = self.mouse_inbound(x, y);
        let needs_redraw = match self.state {
            ButtonState::Normal | ButtonState::Hovered => {
                if self.mouse_inbound(x, y) {
                    self.state = ButtonState::Pressed;
                    true
                } else {
                    false
                }
            }
            ButtonState::Disabled | ButtonState::Pressed => false,
        };
        GuiInteractionResult {
            captured: inbound,
            needs_redraw,
        }
    }

    fn mouse_release(&mut self, x: i32, y: i32) -> (GuiInteractionResult, GuiCommand) {
        if !self.visible {
            return (
                GuiInteractionResult {
                    captured: false,
                    needs_redraw: false,
                },
                GuiCommand::None,
            );
        }
        let inbound = self.mouse_inbound(x, y);
        let (needs_redraw, command) = match self.state {
            ButtonState::Normal => {
                if self.mouse_inbound(x, y) {
                    self.state = ButtonState::Hovered;
                    (true, GuiCommand::None)
                } else {
                    (false, GuiCommand::None)
                }
            }
            ButtonState::Pressed => {
                if self.mouse_inbound(x, y) {
                    self.state = ButtonState::Hovered;
                } else {
                    self.state = ButtonState::Normal;
                }
                (true, self.command)
            }
            ButtonState::Disabled | ButtonState::Hovered => (false, GuiCommand::None),
        };
        (
            GuiInteractionResult {
                captured: inbound,
                needs_redraw,
            },
            command,
        )
    }

    fn enable(&mut self) {
        if let ButtonState::Disabled = self.state {
            self.state = ButtonState::Normal
        }
    }

    fn disable(&mut self) {
        self.state = ButtonState::Disabled;
    }
}

impl GuiInteractionResult {
    fn append(&mut self, other: GuiInteractionResult) {
        self.captured |= other.captured;
        self.needs_redraw |= other.needs_redraw;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GuiCommand {
    None,
    Open,
    Save,
    ZoomIn,
    ZoomOut,
    ShowAll,
    ShowErrors,
    HideErrors,
    NextError,
    PrevError,
}

pub struct Gui {
    buttons: [Button; 9],
    pub height: i32,
}

impl Gui {
    pub fn new(x: i32, y: i32) -> Gui {
        let mut curx = x;
        let mut height = 0;

        let btn_open = Button::new(curx, y, "open", GuiCommand::Open, true);
        curx += btn_open.width + BUTTON_SPACING;
        height = height.max(btn_open.height);

        let mut btn_save = Button::new(curx, y, "save", GuiCommand::Save, true);
        curx += btn_save.width + BUTTON_WIDE_SPACING;
        btn_save.disable();
        height = height.max(btn_save.height);

        let btn_zoomin = Button::new(curx, y, "zoomin", GuiCommand::ZoomIn, true);
        curx += btn_zoomin.width + BUTTON_SPACING;
        height = height.max(btn_zoomin.height);

        let btn_zoomout = Button::new(curx, y, "zoomout", GuiCommand::ZoomOut, true);
        curx += btn_zoomout.width + BUTTON_SPACING;
        height = height.max(btn_zoomout.height);

        let btn_showall = Button::new(curx, y, "showall", GuiCommand::ShowAll, true);
        curx += btn_showall.width + BUTTON_WIDE_SPACING;
        height = height.max(btn_showall.height);

        let btn_shown = Button::new(curx, y, "shown", GuiCommand::HideErrors, true);
        height = height.max(btn_shown.height);

        let btn_hidden = Button::new(curx, y, "hidden", GuiCommand::ShowErrors, false);
        curx += btn_shown.width + BUTTON_SPACING;
        height = height.max(btn_hidden.height);

        let btn_prev = Button::new(curx, y, "prev", GuiCommand::PrevError, true);
        curx += btn_prev.width + BUTTON_SPACING;
        height = height.max(btn_prev.height);

        let btn_next = Button::new(curx, y, "next", GuiCommand::NextError, true);
        height = height.max(btn_next.height);

        Gui {
            buttons: [
                btn_open,
                btn_save,
                btn_zoomin,
                btn_zoomout,
                btn_showall,
                btn_shown,
                btn_hidden,
                btn_prev,
                btn_next,
            ],
            height,
        }
    }

    pub fn draw(&self, pixmap: &mut Pixmap) {
        for btn in &self.buttons {
            btn.draw(pixmap);
        }
    }

    pub fn hide_errors(&mut self) {
        self.buttons[5].visible = false;
        self.buttons[6].visible = true;
        self.buttons[6].state = ButtonState::Hovered;
        self.buttons[7].disable();
        self.buttons[8].disable();
    }

    pub fn show_errors(&mut self) {
        self.buttons[5].visible = true;
        self.buttons[5].state = ButtonState::Hovered;
        self.buttons[6].visible = false;
        self.buttons[7].enable();
        self.buttons[8].enable();
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        let mut total_res = GuiInteractionResult::default();
        for btn in &mut self.buttons {
            total_res.append(btn.mouse_move(x, y));
        }
        total_res
    }

    pub fn mouse_press(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        let mut total_res = GuiInteractionResult::default();
        for btn in &mut self.buttons {
            total_res.append(btn.mouse_press(x, y));
        }
        total_res
    }

    pub fn mouse_release(&mut self, x: i32, y: i32) -> (GuiInteractionResult, GuiCommand) {
        let mut total_res = GuiInteractionResult::default();
        let mut res_command = GuiCommand::None;
        for btn in &mut self.buttons {
            let (res, command) = btn.mouse_release(x, y);
            total_res.append(res);
            match command {
                GuiCommand::None => (),
                _ => res_command = command,
            }
        }

        match res_command {
            GuiCommand::HideErrors => self.hide_errors(),
            GuiCommand::ShowErrors => self.show_errors(),
            _ => (),
        }

        (total_res, res_command)
    }
}
