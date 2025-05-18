mod list;
mod plugin;
mod window_id;
use bevy::reflect::Reflect;
pub use list::*;
pub use plugin::*;
pub use window_id::*;

use bevy::ecs::component::Component;
use bevy::math::IRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum WindowsAppKind {
    Discord,
    NotDiscord,
}

#[derive(Debug, Component, Clone, Reflect)]
pub struct WindowsApp {
    pub id: WindowId,
    pub kind: WindowsAppKind,
    pub bounds: IRect,
    pub title: String,
}
