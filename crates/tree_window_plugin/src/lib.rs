use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiMultipassSchedule};

pub struct TreeWindowPlugin;

impl Plugin for TreeWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_window);
        app.add_systems(UITreeWindowEguiContextPass, ui_tree_window);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct UITreeWindow;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UITreeWindowEguiContextPass;

fn spawn_window(mut commands: Commands) {
    commands.spawn((
        Window {
            title: "UI Tree".to_string(),
            ..default()
        },
        UITreeWindow,
        Name::new("UI Tree Window"),
        EguiMultipassSchedule::new(UITreeWindowEguiContextPass),
    ));
}

fn ui_tree_window(mut window: Query<&mut EguiContext, With<UITreeWindow>>) -> Result {
    let mut ctx = window.single_mut()?;
    egui::Window::new("ui tree").show(ctx.get_mut(), |ui| {
        ui.label("ui tree");
    });
    Ok(())
}
