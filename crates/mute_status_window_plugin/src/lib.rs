use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_egui::EguiMultipassSchedule;
use bevy_inspector_egui::egui;
use ymb_assets::Texture;
use ymb_ipc_plugin::BevyboundIPCMessage;
use ymb_ipc_plugin::IpcWorkerGameboundMessage;
use ymb_ui_automation::MuteButtonState;
use ymb_window_icon_plugin::WindowIcon;

#[derive(Event, Debug, Clone)]
pub enum MuteStatusWindowEvent {
    SpawnWindow,
    DespawnWindow,
    ToggleWindow,
}

pub struct YMBMuteStatusWindowPlugin;

impl Plugin for YMBMuteStatusWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MuteStatusWindowEvent>();
        app.add_systems(Startup, fire_spawn_window_event);
        app.add_systems(Update, handle_ipc_toggle_window_event);
        app.add_systems(Update, handle_spawn_window_event);
        app.add_systems(Update, handle_despawn_window_event);
        app.add_systems(Update, handle_toggle_window_event);
        app.add_systems(MuteStatusWindowEguiContextPass, ui);
    }
}

fn fire_spawn_window_event(mut events: EventWriter<MuteStatusWindowEvent>) {
    events.write(MuteStatusWindowEvent::SpawnWindow);
}

fn handle_ipc_toggle_window_event(
    mut messages: EventReader<IpcWorkerGameboundMessage>,
    mut events: EventWriter<MuteStatusWindowEvent>,
) {
    for msg in messages.read() {
        if let IpcWorkerGameboundMessage::MessageReceived(BevyboundIPCMessage::TrayIconClicked) =
            msg
        {
            events.write(MuteStatusWindowEvent::ToggleWindow);
        }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct MuteStatusWindow;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MuteStatusWindowEguiContextPass;

const DEFAULT_SIZE: (f32, f32) = (320., 100.);

fn handle_spawn_window_event(
    mut events: EventReader<MuteStatusWindowEvent>,
    mut commands: Commands,
    query: Query<Entity, With<MuteStatusWindow>>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        if let MuteStatusWindowEvent::SpawnWindow = event {
            if query.iter().next().is_none() {
                commands.spawn((
                    Window {
                        title: "Mute Status".to_string(),
                        resolution: WindowResolution::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1),
                        ..default()
                    },
                    MuteStatusWindow,
                    Name::new("Mute Status Window"),
                    EguiMultipassSchedule::new(MuteStatusWindowEguiContextPass),
                    WindowIcon::new(asset_server.load(Texture::Icon)),
                ));
                info!("Mute Status window spawned (event)");
            } else {
                info!("Mute Status window already exists, not spawning again (event)");
            }
        }
    }
}

fn handle_despawn_window_event(
    mut events: EventReader<MuteStatusWindowEvent>,
    mut commands: Commands,
    query: Query<Entity, With<MuteStatusWindow>>,
) {
    for event in events.read() {
        if let MuteStatusWindowEvent::DespawnWindow = event {
            if let Some(entity) = query.iter().next() {
                commands.entity(entity).despawn();
                info!("Mute Status window despawned (event)");
            }
        }
    }
}

fn handle_toggle_window_event(
    mut events: EventReader<MuteStatusWindowEvent>,
    mut commands: Commands,
    query: Query<Entity, With<MuteStatusWindow>>,
) {
    for event in events.read() {
        if let MuteStatusWindowEvent::ToggleWindow = event {
            if let Some(entity) = query.iter().next() {
                commands.entity(entity).despawn();
                info!("Mute Status window despawned (toggle event)");
            } else {
                commands.spawn((
                    Window {
                        title: "Mute Status".to_string(),
                        resolution: WindowResolution::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1),
                        ..default()
                    },
                    MuteStatusWindow,
                    Name::new("Mute Status Window"),
                    EguiMultipassSchedule::new(MuteStatusWindowEguiContextPass),
                ));
                // todo: bring to foreground
                info!("Mute Status window spawned (toggle event)");
            }
        }
    }
}

fn ui(world: &mut World) -> Result {
    // Query for the window entity and get its size
    let (window_height, _window_width) = {
        let mut window_query = world.query_filtered::<&Window, With<MuteStatusWindow>>();
        if let Some(window) = window_query.iter(world).next() {
            (window.resolution.height(), window.resolution.width())
        } else {
            (DEFAULT_SIZE.1, DEFAULT_SIZE.0)
        }
    };
    // Set font size proportional to window height (e.g., 40% of height)
    let font_size = window_height * 0.4;
    let mut ctx = world
        .query_filtered::<&mut EguiContext, With<MuteStatusWindow>>()
        .single_mut(world)?
        .clone();
    let mute_state = world
        .query_filtered::<&MuteButtonState, ()>()
        .iter(world)
        .next()
        .cloned()
        .unwrap_or(MuteButtonState::NotMuted);
    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        let (text, color) = match mute_state {
            MuteButtonState::Muted => ("You are muted btw.", egui::Color32::RED),
            _ => ("You are not muted.", egui::Color32::GREEN),
        };
        let style = ui.style_mut();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(font_size, egui::FontFamily::Proportional),
        );
        ui.colored_label(color, egui::RichText::new(text).heading());
    });
    Ok(())
}
