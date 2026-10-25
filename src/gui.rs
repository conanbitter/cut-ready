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
}

pub struct GuiInteractionResult {
    pub captured: bool,
    pub needs_redraw: bool,
}

impl Button {
    fn new(x: i32, y: i32, name: &str, visible: bool) -> Button {
        let image_normal = Pixmap::load_png(format!("WIP/{}_normal.png", name)).unwrap();
        let image_hovered = Pixmap::load_png(format!("WIP/{}_hovered.png", name)).unwrap();
        let image_pressed = Pixmap::load_png(format!("WIP/{}_pressed.png", name)).unwrap();
        let image_disabled = Some(Pixmap::load_png(format!("WIP/{}_disabled.png", name)).unwrap());
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
        pixmap.draw_pixmap(10, 10, image.as_ref(), &GUI_PAINT, GUI_TRANSFORM, None);
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

    fn mouse_release(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        if !self.visible {
            return GuiInteractionResult {
                captured: false,
                needs_redraw: false,
            };
        }
        let inbound = self.mouse_inbound(x, y);
        let needs_redraw = match self.state {
            ButtonState::Normal => {
                if self.mouse_inbound(x, y) {
                    self.state = ButtonState::Hovered;
                    true
                } else {
                    false
                }
            }
            ButtonState::Pressed => {
                if self.mouse_inbound(x, y) {
                    self.state = ButtonState::Hovered;
                } else {
                    self.state = ButtonState::Normal;
                }
                true
            }
            ButtonState::Disabled | ButtonState::Hovered => false,
        };
        GuiInteractionResult {
            captured: inbound,
            needs_redraw,
        }
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

pub struct Gui {
    button: Button,
}

impl Gui {
    pub fn new() -> Gui {
        Gui {
            button: Button::new(10, 10, "button", false),
        }
    }

    pub fn draw(&self, pixmap: &mut Pixmap) {
        self.button.draw(pixmap);
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        self.button.mouse_move(x, y)
    }

    pub fn mouse_press(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        self.button.mouse_press(x, y)
    }

    pub fn mouse_release(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        self.button.mouse_release(x, y)
    }
}
