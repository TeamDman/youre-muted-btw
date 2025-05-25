use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiContext;
use bevy_egui::EguiMultipassSchedule;
use bevy_egui::egui;

pub struct TreeWindowPlugin;

impl Plugin for TreeWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_window);
        app.add_systems(UITreeWindowEguiContextPass, ui_tree_window);
    }
}
const DEFAULT_SIZE: (f32, f32) = (320., 160.);

#[derive(Debug, Component, Reflect)]
pub struct UITreeWindow;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UITreeWindowEguiContextPass;

fn spawn_window(mut commands: Commands) {
    commands.spawn((
        Window {
            title: "UI Tree".to_string(),
            resolution: WindowResolution::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1),
            ..default()
        },
        UITreeWindow,
        Name::new("UI Tree Window"),
        EguiMultipassSchedule::new(UITreeWindowEguiContextPass),
    ));
}

fn ui_tree_window(mut window: Query<&mut EguiContext, With<UITreeWindow>>) -> Result {
    let mut ctx = window.single_mut()?;
    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            ui.label("ui tree");
        });
    });
    Ok(())
}
