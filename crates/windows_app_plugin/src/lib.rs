mod list;
mod plugin;
mod window_id;
pub use list::*;
pub use plugin::*;
pub use window_id::*;

use bevy::ecs::component::Component;
use bevy::math::IRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowsAppKind {
    Discord,
    NotDiscord,
}

#[derive(Debug, Component, Clone)]
pub struct WindowsApp {
    pub id: WindowId,
    pub kind: WindowsAppKind,
    pub bounds: IRect,
    pub title: String,
}
