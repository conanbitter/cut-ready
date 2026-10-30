use std::time::{Duration, Instant};

use tiny_skia::{BlendMode, FilterQuality, Pixmap, PixmapPaint, Transform};

pub const GUI_PAINT: PixmapPaint = PixmapPaint {
    opacity: 1.0,
    blend_mode: BlendMode::Source,
    quality: FilterQuality::Nearest,
};

const BUTTON_SPACING: i32 = 2;
const BUTTON_WIDE_SPACING: i32 = 10;

const TOOLTIP_FADEIN_DURATION: Duration = Duration::from_millis(700);
const TOOLTIP_FADEOUT_DURATION: Duration = Duration::from_millis(200);

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

const TTIMG: &[&[&[u8]]] = &[
    // EN
    &[
        include_bytes!("../assets/tooltips/en/open.png"),
        include_bytes!("../assets/tooltips/en/save.png"),
        include_bytes!("../assets/tooltips/en/zoomin.png"),
        include_bytes!("../assets/tooltips/en/zoomout.png"),
        include_bytes!("../assets/tooltips/en/showall.png"),
        include_bytes!("../assets/tooltips/en/shown.png"),
        include_bytes!("../assets/tooltips/en/hidden.png"),
        include_bytes!("../assets/tooltips/en/prev.png"),
        include_bytes!("../assets/tooltips/en/next.png"),
    ],
    // RU
    &[
        include_bytes!("../assets/tooltips/ru/open.png"),
        include_bytes!("../assets/tooltips/ru/save.png"),
        include_bytes!("../assets/tooltips/ru/zoomin.png"),
        include_bytes!("../assets/tooltips/ru/zoomout.png"),
        include_bytes!("../assets/tooltips/ru/showall.png"),
        include_bytes!("../assets/tooltips/ru/shown.png"),
        include_bytes!("../assets/tooltips/ru/hidden.png"),
        include_bytes!("../assets/tooltips/ru/prev.png"),
        include_bytes!("../assets/tooltips/ru/next.png"),
    ],
];

#[derive(Copy, Clone, PartialEq)]
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
    redraw_needed: bool,
}

#[derive(PartialEq, PartialOrd, Default)]
pub enum RedrawLevel {
    #[default]
    None,
    GuiOnly,
    Full,
}

#[derive(Default)]
pub struct GuiInteractionResult {
    pub captured: bool,
    pub needs_redraw: RedrawLevel,
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
            redraw_needed: true,
        }
    }

    fn draw(&mut self, pixmap: &mut Pixmap, forced: bool) {
        if !self.visible {
            return;
        }
        if self.redraw_needed || forced {
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
            pixmap.draw_pixmap(self.x, self.y, image.as_ref(), &GUI_PAINT, Transform::identity(), None);
            self.redraw_needed = false;
        }
    }

    fn mouse_inbound(&self, x: i32, y: i32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn mouse_move(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        let inbound = self.mouse_inbound(x, y);
        let old_state = self.state;
        match self.state {
            ButtonState::Normal if inbound => self.state = ButtonState::Hovered,
            ButtonState::Hovered | ButtonState::Pressed if !inbound => self.state = ButtonState::Normal,
            _ => (),
        };
        self.redraw_needed = self.redraw_needed || self.state != old_state;
        inbound
    }

    fn mouse_press(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        let inbound = self.mouse_inbound(x, y);
        let old_state = self.state;
        match self.state {
            ButtonState::Normal | ButtonState::Hovered if inbound => self.state = ButtonState::Pressed,
            _ => (),
        };
        self.redraw_needed = self.redraw_needed || self.state != old_state;
        inbound
    }

    fn mouse_release(&mut self, x: i32, y: i32) -> (bool, GuiCommand) {
        if !self.visible {
            return (false, GuiCommand::None);
        }
        let inbound = self.mouse_inbound(x, y);
        let old_state = self.state;
        let command = match self.state {
            ButtonState::Normal if inbound => {
                self.state = ButtonState::Hovered;
                GuiCommand::None
            }
            ButtonState::Pressed => {
                if inbound {
                    self.state = ButtonState::Hovered;
                } else {
                    self.state = ButtonState::Normal;
                }
                self.command
            }
            _ => GuiCommand::None,
        };
        self.redraw_needed = self.redraw_needed || self.state != old_state;
        (inbound, command)
    }

    fn enable(&mut self) {
        if let ButtonState::Disabled = self.state {
            self.state = ButtonState::Normal
        }
    }

    fn disable(&mut self) {
        self.state = ButtonState::Disabled;
    }

    fn is_disabled(&self) -> bool {
        matches!(self.state, ButtonState::Disabled)
    }
}

/*
impl GuiInteractionResult {
    fn append(&mut self, other: GuiInteractionResult) {
        self.captured |= other.captured;
        self.needs_redraw |= other.needs_redraw;
    }
}
*/

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

enum TooltipState {
    Showing,
    Hidden,
    FadeIn,
    FadeOut,
}

pub struct Gui {
    buttons: [Button; 9],
    tooltips: Vec<Pixmap>,
    pub height: i32,
    pub timer: Option<Instant>,
    tooltip_button: Option<usize>,
    tooltip_state: TooltipState,
    tooltip_x: i32,
    tooltip_y: i32,
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

        let tooltips = match t!("tooltips").as_ref() {
            "en" => TTIMG[0],
            "ru" => TTIMG[1],
            _ => TTIMG[0],
        };

        let tooltips: Vec<Pixmap> = tooltips.iter().map(|data| Pixmap::decode_png(data).unwrap()).collect();

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
            timer: None,
            tooltip_button: None,
            tooltip_state: TooltipState::Hidden,
            tooltips,
            tooltip_x: 0,
            tooltip_y: 0,
        }
    }

    pub fn draw(&mut self, pixmap: &mut Pixmap, forced: bool) {
        for btn in &mut self.buttons {
            btn.draw(pixmap, forced);
        }

        if matches!(self.tooltip_state, TooltipState::Showing)
            && let Some(btn) = self.tooltip_button
        {
            pixmap.draw_pixmap(
                self.tooltip_x,
                self.tooltip_y,
                self.tooltips[btn].as_ref(),
                &GUI_PAINT,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn hide_errors(&mut self, permanent: bool, x: i32, y: i32) {
        self.buttons[5].visible = false;
        self.buttons[6].visible = true;
        if permanent {
            self.buttons[6].state = ButtonState::Disabled;
        } else {
            self.buttons[6].state = if self.buttons[6].mouse_inbound(x, y) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            }
        }
        self.buttons[7].disable();
        self.buttons[8].disable();

        self.buttons[5].redraw_needed = true;
        self.buttons[6].redraw_needed = true;
        self.buttons[7].redraw_needed = true;
        self.buttons[8].redraw_needed = true;
    }

    pub fn show_errors(&mut self, permanent: bool, x: i32, y: i32) {
        self.buttons[5].visible = true;
        self.buttons[5].state = if self.buttons[5].mouse_inbound(x, y) {
            ButtonState::Hovered
        } else {
            ButtonState::Normal
        };
        self.buttons[6].visible = false;
        if permanent {
            self.buttons[6].state = ButtonState::Normal;
        }
        self.buttons[7].enable();
        self.buttons[8].enable();

        self.buttons[5].redraw_needed = true;
        self.buttons[6].redraw_needed = true;
        self.buttons[7].redraw_needed = true;
        self.buttons[8].redraw_needed = true;
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        let mut total_captured = false;
        let mut redraw_needed = RedrawLevel::None;
        let mut hovered: Option<usize> = None;
        for (i, btn) in self.buttons.iter_mut().enumerate() {
            let captured = btn.mouse_move(x, y);
            total_captured = total_captured || captured;
            if btn.redraw_needed {
                redraw_needed = RedrawLevel::GuiOnly
            }
            if captured && !btn.is_disabled() {
                hovered = Some(i);
            }
        }

        if let Some(hovered_index) = hovered {
            if hovered != self.tooltip_button && self.on_enter(hovered_index) {
                redraw_needed = RedrawLevel::Full;
            }
        } else {
            if hovered != self.tooltip_button {
                self.on_leave(false);
            }
        }

        GuiInteractionResult {
            captured: total_captured,
            needs_redraw: redraw_needed,
        }
    }

    pub fn mouse_press(&mut self, x: i32, y: i32) -> GuiInteractionResult {
        let mut total_captured = false;
        let mut redraw_needed = RedrawLevel::None;
        for btn in &mut self.buttons {
            let captured = btn.mouse_press(x, y);
            total_captured = total_captured || captured;
            if btn.redraw_needed {
                redraw_needed = RedrawLevel::GuiOnly;
            }
        }
        if self.tooltip_button.is_some() && self.on_leave(true) {
            redraw_needed = RedrawLevel::Full;
        }
        GuiInteractionResult {
            captured: total_captured,
            needs_redraw: redraw_needed,
        }
    }

    pub fn mouse_release(&mut self, x: i32, y: i32) -> (GuiInteractionResult, GuiCommand) {
        let mut total_captured = false;
        let mut redraw_needed = false;
        let mut res_command = GuiCommand::None;
        for btn in &mut self.buttons {
            let (captured, command) = btn.mouse_release(x, y);
            total_captured = total_captured || captured;
            redraw_needed = redraw_needed || btn.redraw_needed;
            match command {
                GuiCommand::None => (),
                _ => res_command = command,
            }
        }

        match res_command {
            GuiCommand::HideErrors => self.hide_errors(false, x, y),
            GuiCommand::ShowErrors => self.show_errors(false, x, y),
            _ => (),
        }

        (
            GuiInteractionResult {
                captured: total_captured,
                needs_redraw: if redraw_needed {
                    RedrawLevel::GuiOnly
                } else {
                    RedrawLevel::None
                },
            },
            res_command,
        )
    }

    pub fn mouse_leaved(&mut self) -> bool {
        let mut redraw_needed = false;
        for btn in &mut self.buttons {
            match btn.state {
                ButtonState::Hovered | ButtonState::Pressed => {
                    btn.state = ButtonState::Normal;
                    btn.redraw_needed = true;
                    redraw_needed = true;
                }
                _ => (),
            }
        }
        self.on_leave(false);
        redraw_needed
    }

    pub fn enable_saving(&mut self) {
        self.buttons[1].enable();
        self.buttons[1].redraw_needed = true;
    }

    pub fn disable_saving(&mut self) {
        self.buttons[1].disable();
        self.buttons[1].redraw_needed = true;
    }

    pub fn toggle_errors(&mut self, x: i32, y: i32) -> bool {
        if let ButtonState::Disabled = self.buttons[6].state {
            return false;
        }

        if self.buttons[6].visible {
            self.show_errors(false, x, y);
            true
        } else {
            self.hide_errors(false, x, y);
            false
        }
    }

    fn set_timer(&mut self, duration: Duration) {
        self.timer = Some(Instant::now() + duration);
    }

    pub fn reset_timer(&mut self) {
        self.timer = None
    }

    pub fn on_timer(&mut self) -> RedrawLevel {
        let mut needs_redraw = RedrawLevel::None;
        match self.tooltip_state {
            TooltipState::FadeIn => {
                if let Some(btn) = self.tooltip_button {
                    self.tooltip_state = TooltipState::Showing;
                    self.tooltip_x = self.buttons[btn].x;
                    self.tooltip_y = self.buttons[btn].y + self.buttons[btn].height + 2;
                    needs_redraw = RedrawLevel::GuiOnly;
                }
            }
            TooltipState::FadeOut => {
                self.tooltip_state = TooltipState::Hidden;
                needs_redraw = RedrawLevel::Full;
            }
            _ => (),
        }
        needs_redraw
    }

    fn on_enter(&mut self, btn: usize) -> bool {
        let mut needs_redraw = false;
        match self.tooltip_state {
            TooltipState::Showing if Some(btn) != self.tooltip_button => {
                self.tooltip_x = self.buttons[btn].x;
                self.tooltip_y = self.buttons[btn].y + self.buttons[btn].height + 2;
                needs_redraw = true;
            }
            TooltipState::Hidden => {
                self.tooltip_state = TooltipState::FadeIn;
                self.set_timer(TOOLTIP_FADEIN_DURATION);
            }
            TooltipState::FadeIn => self.set_timer(TOOLTIP_FADEIN_DURATION),
            TooltipState::FadeOut => {
                if Some(btn) != self.tooltip_button {
                    self.tooltip_x = self.buttons[btn].x;
                    self.tooltip_y = self.buttons[btn].y + self.buttons[btn].height + 2;
                    needs_redraw = true;
                }
                self.tooltip_state = TooltipState::Showing;
                self.reset_timer();
            }
            _ => (),
        }
        self.tooltip_button = Some(btn);
        needs_redraw
    }

    fn on_leave(&mut self, immediate: bool) -> bool {
        self.tooltip_button = None;
        let mut redraw_needed = false;

        if immediate {
            if !matches!(self.tooltip_state, TooltipState::Hidden) {
                redraw_needed = true;
            }
            self.tooltip_state = TooltipState::Hidden;
            self.reset_timer();
        }

        match self.tooltip_state {
            TooltipState::Showing => {
                self.tooltip_state = TooltipState::FadeOut;
                self.set_timer(TOOLTIP_FADEOUT_DURATION);
            }
            TooltipState::FadeIn => {
                self.tooltip_state = TooltipState::Hidden;
                self.reset_timer();
            }
            _ => (),
        };
        redraw_needed
    }
}
