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
use ymb_ipc_plugin::IpcWorkerGameboundMessage;
use ymb_ipc_plugin::BevyboundIPCMessage;

#[derive(Event, Debug, Clone)]
pub enum WorldInspectorWindowEvent {
    SpawnWindow,
    DespawnWindow,
    ToggleWindow,
}

pub struct YMBWorldInspectorPlugin;

impl Plugin for YMBWorldInspectorPlugin {
    fn build(&self, app: &mut App) {
        check_plugins(app, "WorldInspectorPlugin");
        app.add_event::<WorldInspectorWindowEvent>();
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Startup, fire_spawn_window_event);
        app.add_systems(WorldInspectorWindowEguiContextPass, ui);
        app.add_systems(Update, handle_ipc_toggle_window_event);
        app.add_systems(Update, handle_spawn_window_event);
        app.add_systems(Update, handle_despawn_window_event);
        app.add_systems(Update, handle_toggle_window_event);
    }
}

fn fire_spawn_window_event(mut events: EventWriter<WorldInspectorWindowEvent>) {
    events.write(WorldInspectorWindowEvent::SpawnWindow);
}

fn handle_ipc_toggle_window_event(
    mut messages: EventReader<IpcWorkerGameboundMessage>,
    mut events: EventWriter<WorldInspectorWindowEvent>,
) {
    for msg in messages.read() {
        if let IpcWorkerGameboundMessage::MessageReceived(BevyboundIPCMessage::ShowWorldInspector) = msg {
            events.write(WorldInspectorWindowEvent::ToggleWindow);
        }
    }
}

fn handle_spawn_window_event(
    mut events: EventReader<WorldInspectorWindowEvent>,
    mut commands: Commands,
    query: Query<Entity, With<WorldInspectorWindow>>,
) {
    for event in events.read() {
        if let WorldInspectorWindowEvent::SpawnWindow = event {
            if query.iter().next().is_none() {
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
                info!("World Inspector window spawned (event)");
            } else {
                info!("World Inspector window already exists, not spawning again (event)");
            }
        }
    }
}

fn handle_despawn_window_event(
    mut events: EventReader<WorldInspectorWindowEvent>,
    mut commands: Commands,
    query: Query<Entity, With<WorldInspectorWindow>>,
) {
    for event in events.read() {
        if let WorldInspectorWindowEvent::DespawnWindow = event {
            if let Some(entity) = query.iter().next() {
                commands.entity(entity).despawn();
                info!("World Inspector window despawned (event)");
            }
        }
    }
}

fn handle_toggle_window_event(
    mut events: EventReader<WorldInspectorWindowEvent>,
    mut commands: Commands,
    query: Query<Entity, With<WorldInspectorWindow>>,
) {
    for event in events.read() {
        if let WorldInspectorWindowEvent::ToggleWindow = event {
            if let Some(entity) = query.iter().next() {
                commands.entity(entity).despawn();
                info!("World Inspector window despawned (toggle event)");
            } else {
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
                info!("World Inspector window spawned (toggle event)");
            }
        }
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
