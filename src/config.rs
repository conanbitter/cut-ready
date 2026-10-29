use anyhow::anyhow;
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::LazyLock,
};
use tinyjson::JsonValue;

use crate::blueprint::LINEWIDTH_BIAS;

#[derive(Debug)]
pub struct ConfigColors {
    pub main: HashSet<i32>,
    pub cuts: HashSet<i32>,
    pub marks: HashSet<i32>,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfWindow {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct ConfLineWidths {
    pub main: f32,
    pub thick: f32,
    pub thin: f32,
    pub error: f32,
}

#[derive(Debug)]
pub struct Configuration {
    pub colors: ConfigColors,
    pub window: ConfWindow,
    pub line_widths: ConfLineWidths,
    pub error_size: f32,
}

impl Default for Configuration {
    fn default() -> Self {
        let colors = ConfigColors {
            main: HashSet::from([0x007FFF, 0x0080FF, 0x0000FF]),
            cuts: HashSet::from([0x008181, 0x009696, 0x006868]),
            marks: HashSet::from([0]),
        };

        Configuration {
            colors,
            window: ConfWindow { width: 0, height: 0 },
            line_widths: ConfLineWidths {
                main: 2.0 - LINEWIDTH_BIAS,
                thick: 3.0 - LINEWIDTH_BIAS,
                thin: 1.0 - LINEWIDTH_BIAS,
                error: 3.0 - LINEWIDTH_BIAS,
            },
            error_size: 15.0,
        }
    }
}

impl Configuration {
    fn new() -> Configuration {
        let mut config = Configuration::default();
        let _ = config.load();

        //println!("{:?}", config);

        config
    }

    fn load(&mut self) -> anyhow::Result<()> {
        let file = fs::read_to_string("config.json")?;
        let file: JsonValue = file.parse()?;
        let file: &HashMap<_, _> = file.get().ok_or(anyhow!("Wrong config type"))?;

        //Colors
        if let Some(JsonValue::Object(colors)) = file.get("colors") {
            if let Some(main) = colors.get("main")
                && let Some(colors) = json_to_color_set(main)
            {
                self.colors.main = colors;
            }

            if let Some(cuts) = colors.get("cuts")
                && let Some(colors) = json_to_color_set(cuts)
            {
                self.colors.cuts = colors;
            }

            if let Some(marks) = colors.get("marks")
                && let Some(colors) = json_to_color_set(marks)
            {
                self.colors.marks = colors;
            }
        }

        // Window
        if let Some(JsonValue::Object(window)) = file.get("window") {
            if let Some(width) = window.get("width")
                && let JsonValue::Number(width) = width
            {
                self.window.width = *width as i32;
            }

            if let Some(height) = window.get("height")
                && let JsonValue::Number(height) = height
            {
                self.window.height = *height as i32;
            }
        }

        //Line widths
        if let Some(JsonValue::Object(lws)) = file.get("line_widths") {
            if let Some(width) = lws.get("main")
                && let JsonValue::Number(width) = width
            {
                self.line_widths.main = *width as f32 - LINEWIDTH_BIAS;
            }

            if let Some(width) = lws.get("thick")
                && let JsonValue::Number(width) = width
            {
                self.line_widths.thick = *width as f32 - LINEWIDTH_BIAS;
            }

            if let Some(width) = lws.get("thin")
                && let JsonValue::Number(width) = width
            {
                self.line_widths.thin = *width as f32 - LINEWIDTH_BIAS;
            }

            if let Some(width) = lws.get("error")
                && let JsonValue::Number(width) = width
            {
                self.line_widths.error = *width as f32 - LINEWIDTH_BIAS;
            }
        }

        // Errors
        if let Some(size) = file.get("error_size")
            && let JsonValue::Number(size) = size
        {
            self.error_size = *size as f32 / 2.0;
        }

        Ok(())
    }
}

fn json_to_color(value: &JsonValue) -> Option<i32> {
    match value {
        JsonValue::Number(num) => Some(*num as i32),
        JsonValue::String(st) if st.starts_with('#') && st.len() == 7 => i32::from_str_radix(&st[1..], 16).ok(),
        JsonValue::Array(arr) if arr.len() == 3 => {
            if let JsonValue::Number(r) = arr[0]
                && let JsonValue::Number(g) = arr[1]
                && let JsonValue::Number(b) = arr[2]
            {
                Some((r as i32 | 0xFF) << 16 | (g as i32 | 0xFF) << 8 | (b as i32 | 0xFF))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn json_to_color_set(value: &JsonValue) -> Option<HashSet<i32>> {
    match value {
        JsonValue::Array(arr) => {
            let res: HashSet<i32> = arr.iter().filter_map(json_to_color).collect();
            if res.is_empty() { None } else { Some(res) }
        }
        _ => json_to_color(value).map(|value| HashSet::from([value])),
    }
}

pub static CONFIG: LazyLock<Configuration> = LazyLock::new(Configuration::new);
