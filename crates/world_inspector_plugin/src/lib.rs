use bevy::app::MainSchedulePlugin;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_egui::EguiMultipassSchedule;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;
use bevy_inspector_egui::egui;

pub struct YMBWorldInspectorPlugin;

impl Plugin for YMBWorldInspectorPlugin {
    fn build(&self, app: &mut App) {
        check_plugins(app, "WorldInspectorPlugin");
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Startup, spawn_window);
        app.add_systems(WorldInspectorWindowEguiContextPass, ui);
    }
}

/// Copied from bevy-inspector-egui/src/quick.rs
fn check_plugins(app: &App, name: &str) {
    if !app.is_plugin_added::<MainSchedulePlugin>() {
        panic!(
            r#"`{name}` should be added after the default plugins:
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {{ .. }})
        .add_plugins({name}::default())
            "#,
        );
    }

    if !app.is_plugin_added::<EguiPlugin>() {
        panic!(
            r#"`{name}` needs to be added after `EguiPlugin`:
        .add_plugins(EguiPlugin {{ enable_multipass_for_primary_context: true }})
        .add_plugins({name}::default())
            "#,
        );
    }
}

#[derive(Debug, Component, Reflect)]
pub struct WorldInspectorWindow;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorldInspectorWindowEguiContextPass;

fn spawn_window(mut commands: Commands) {
    commands.spawn((
        Window {
            title: "World Inspector".to_string(),
            resolution: WindowResolution::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1),
            ..default()
        },
        WorldInspectorWindow,
        Name::new("World Inspector Window"),
        EguiMultipassSchedule::new(WorldInspectorWindowEguiContextPass),
    ));
}
const DEFAULT_SIZE: (f32, f32) = (320., 160.);

fn ui(world: &mut World) -> Result {
    let mut ctx = world
        .query_filtered::<&mut EguiContext, With<WorldInspectorWindow>>()
        .single_mut(world)?
        .clone();
    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            bevy_inspector::ui_for_world(world, ui);
            ui.allocate_space(ui.available_size());
        });
    });
    Ok(())
}
