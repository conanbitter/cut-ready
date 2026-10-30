use std::{error::Error, fmt, num::NonZeroU32};

#[derive(Debug)]
pub enum AppError {
    PathIsEmpty,
    SurfaceNew,
    SurfaceResize(NonZeroU32, NonZeroU32),
    SurfaceDraw,
    InkerNew,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::PathIsEmpty => write!(f, "Path is empty"),
            AppError::SurfaceNew => write!(f, "Window surface can not be created"),
            AppError::SurfaceResize(w, h) => write!(f, "Can not resize window surface to {}x{}", w, h),
            AppError::SurfaceDraw => write!(f, "Can't draw on window surface"),
            AppError::InkerNew => write!(f, "Inker canvas can not be created"),
        }
    }
}

impl Error for AppError {}

pub fn finish_path(builder: tiny_skia::PathBuilder) -> anyhow::Result<tiny_skia::Path> {
    match builder.finish() {
        Some(path) => Ok(path),
        None => Err(AppError::PathIsEmpty.into()),
    }
}

pub fn default_rect() -> tiny_skia::Rect {
    tiny_skia::Rect::from_ltrb(0.0, 0.0, 1.0, 1.0).unwrap() // constant
}
