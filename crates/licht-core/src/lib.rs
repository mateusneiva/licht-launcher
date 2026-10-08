//! Minecraft launcher core, independent of Tauri.

mod error;

pub use error::CoreError;

pub type Result<T> = std::result::Result<T, CoreError>;
