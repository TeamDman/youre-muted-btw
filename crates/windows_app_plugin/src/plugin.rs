use crate::WindowId;
use crate::WindowsApp;
use crate::WindowsAppKind;
use crate::get_apps;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

pub struct WindowsAppPlugin;

impl Plugin for WindowsAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<ThreadboundMessage, GameboundMessage, ()> {
                name: "WindowsAppWorker".to_string(),
                handle_threadbound_message,
                ..default()
            },
        });
        app.init_resource::<WindowsAppListPluginConfig>();
        app.register_type::<WindowsAppListPluginConfig>();
        app.register_type::<WindowsApp>();
        app.register_type::<WindowId>();
        app.register_type::<WindowsAppKind>();
        app.add_systems(Update, handle_gamebound_messages);
        app.add_systems(Update, tick);
        app.add_systems(
            Startup,
            |mut threadbound_messages: EventWriter<ThreadboundMessage>| {
                threadbound_messages.write(ThreadboundMessage::Gather);
            },
        );
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct WindowsAppListPluginConfig {
    pub refresh_interval: Timer,
}
impl Default for WindowsAppListPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum ThreadboundMessage {
    Gather,
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum GameboundMessage {
    Latest(HashMap<WindowId, WindowsApp>),
}

fn handle_threadbound_message(
    msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    _state: &mut (),
) -> Result<()> {
    match msg {
        ThreadboundMessage::Gather => {
            info!("Gathering Windows apps");
            let apps = get_apps()?;
            reply_tx.send(GameboundMessage::Latest(apps))?;
        }
    }
    Ok(())
}

fn handle_gamebound_messages(
    mut commands: Commands,
    mut messages: EventReader<GameboundMessage>,
    mut apps: Query<(Entity, &WindowsApp)>,
) {
    if let Some(GameboundMessage::Latest(msg)) = messages.read().last() {
        for (entity, app) in apps.iter_mut() {
            if let Some(fresh_app) = msg.get(&app.id) {
                commands.entity(entity).insert(fresh_app.clone());
            } else {
                commands.entity(entity).despawn();
            }
        }
        for (id, app) in msg.iter() {
            if !apps.iter().any(|(_, a)| a.id == *id) {
                commands.spawn((
                    app.clone(),
                    Name::new(format!("WindowsApp: {:?}", app.title)),
                ));
            }
        }
    }
}

fn tick(
    mut config: ResMut<WindowsAppListPluginConfig>,
    time: Res<Time>,
    mut threadbound_messages: EventWriter<ThreadboundMessage>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    threadbound_messages.write(ThreadboundMessage::Gather);
}
