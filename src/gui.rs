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

struct ButtonImages {
    normal: &'static [u8],
    hovered: &'static [u8],
    pressed: &'static [u8],
    disabled: Option<&'static [u8]>,
}

const BTNIMG_OPEN: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/open_normal.png"),
    hovered: include_bytes!("../assets/buttons/open_hovered.png"),
    pressed: include_bytes!("../assets/buttons/open_pressed.png"),
    disabled: None,
};

const BTNIMG_SAVE: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/save_normal.png"),
    hovered: include_bytes!("../assets/buttons/save_hovered.png"),
    pressed: include_bytes!("../assets/buttons/save_pressed.png"),
    disabled: Some(include_bytes!("../assets/buttons/save_disabled.png")),
};

const BTNIMG_ZOOMIN: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/zoomin_normal.png"),
    hovered: include_bytes!("../assets/buttons/zoomin_hovered.png"),
    pressed: include_bytes!("../assets/buttons/zoomin_pressed.png"),
    disabled: None,
};

const BTNIMG_ZOOMOUT: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/zoomout_normal.png"),
    hovered: include_bytes!("../assets/buttons/zoomout_hovered.png"),
    pressed: include_bytes!("../assets/buttons/zoomout_pressed.png"),
    disabled: None,
};

const BTNIMG_SHOWALL: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/showall_normal.png"),
    hovered: include_bytes!("../assets/buttons/showall_hovered.png"),
    pressed: include_bytes!("../assets/buttons/showall_pressed.png"),
    disabled: None,
};

const BTNIMG_SHOWN: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/shown_normal.png"),
    hovered: include_bytes!("../assets/buttons/shown_hovered.png"),
    pressed: include_bytes!("../assets/buttons/shown_pressed.png"),
    disabled: None,
};

const BTNIMG_HIDDEN: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/hidden_normal.png"),
    hovered: include_bytes!("../assets/buttons/hidden_hovered.png"),
    pressed: include_bytes!("../assets/buttons/hidden_pressed.png"),
    disabled: Some(include_bytes!("../assets/buttons/hidden_disabled.png")),
};

const BTNIMG_NEXT: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/next_normal.png"),
    hovered: include_bytes!("../assets/buttons/next_hovered.png"),
    pressed: include_bytes!("../assets/buttons/next_pressed.png"),
    disabled: Some(include_bytes!("../assets/buttons/next_disabled.png")),
};

const BTNIMG_PREV: ButtonImages = ButtonImages {
    normal: include_bytes!("../assets/buttons/prev_normal.png"),
    hovered: include_bytes!("../assets/buttons/prev_hovered.png"),
    pressed: include_bytes!("../assets/buttons/prev_pressed.png"),
    disabled: Some(include_bytes!("../assets/buttons/prev_disabled.png")),
};

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
    fn new(x: i32, y: i32, images: &ButtonImages, command: GuiCommand, visible: bool) -> Button {
        let image_normal = Pixmap::decode_png(images.normal).unwrap();
        let image_hovered = Pixmap::decode_png(images.hovered).unwrap();
        let image_pressed = Pixmap::decode_png(images.pressed).unwrap();
        let image_disabled = match images.disabled {
            Some(image) => Pixmap::decode_png(image).ok(),
            None => None,
        };
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

        let btn_open = Button::new(curx, y, &BTNIMG_OPEN, GuiCommand::Open, true);
        curx += btn_open.width + BUTTON_SPACING;
        height = height.max(btn_open.height);

        let mut btn_save = Button::new(curx, y, &BTNIMG_SAVE, GuiCommand::Save, true);
        curx += btn_save.width + BUTTON_WIDE_SPACING;
        btn_save.disable();
        height = height.max(btn_save.height);

        let btn_zoomin = Button::new(curx, y, &BTNIMG_ZOOMIN, GuiCommand::ZoomIn, true);
        curx += btn_zoomin.width + BUTTON_SPACING;
        height = height.max(btn_zoomin.height);

        let btn_zoomout = Button::new(curx, y, &BTNIMG_ZOOMOUT, GuiCommand::ZoomOut, true);
        curx += btn_zoomout.width + BUTTON_SPACING;
        height = height.max(btn_zoomout.height);

        let btn_showall = Button::new(curx, y, &BTNIMG_SHOWALL, GuiCommand::ShowAll, true);
        curx += btn_showall.width + BUTTON_WIDE_SPACING;
        height = height.max(btn_showall.height);

        let btn_shown = Button::new(curx, y, &BTNIMG_SHOWN, GuiCommand::HideErrors, false);
        height = height.max(btn_shown.height);

        let mut btn_hidden = Button::new(curx, y, &BTNIMG_HIDDEN, GuiCommand::ShowErrors, true);
        curx += btn_shown.width + BUTTON_SPACING;
        btn_hidden.disable();
        height = height.max(btn_hidden.height);

        let mut btn_prev = Button::new(curx, y, &BTNIMG_PREV, GuiCommand::PrevError, true);
        curx += btn_prev.width + BUTTON_SPACING;
        btn_prev.disable();
        height = height.max(btn_prev.height);

        let mut btn_next = Button::new(curx, y, &BTNIMG_NEXT, GuiCommand::NextError, true);
        height = height.max(btn_next.height);
        btn_next.disable();

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

    pub fn hide_errors(&mut self, permanent: bool) {
        self.buttons[5].visible = false;
        self.buttons[6].visible = true;
        if permanent {
            self.buttons[6].state = ButtonState::Disabled;
        } else {
            self.buttons[6].state = ButtonState::Hovered;
        }
        self.buttons[7].disable();
        self.buttons[8].disable();
    }

    pub fn show_errors(&mut self, permanent: bool) {
        self.buttons[5].visible = true;
        self.buttons[5].state = ButtonState::Hovered;
        self.buttons[6].visible = false;
        if permanent {
            self.buttons[6].state = ButtonState::Normal;
        }
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
            GuiCommand::HideErrors => self.hide_errors(false),
            GuiCommand::ShowErrors => self.show_errors(false),
            _ => (),
        }

        (total_res, res_command)
    }

    pub fn enable_saving(&mut self) {
        self.buttons[1].enable();
    }

    pub fn disable_saving(&mut self) {
        self.buttons[1].disable();
    }
}
