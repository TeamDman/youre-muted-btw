use bevy::prelude::*;
use bevy::state::commands;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_ui_automation::AncestryTree;
use ymb_ui_automation::ElementInfo;
use ymb_ui_automation::gather_tree_from_position;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

pub struct ElementInfoPlugin;

impl Plugin for ElementInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<ThreadboundMessage, GameboundMessage, WorkerState> {
                name: "ElementInfoPluginWorker".to_string(),
                handle_threadbound_message,
                ..default()
            },
        });
        app.add_systems(Update, handle_gamebound_messages);
        app.add_systems(Update, tick_worker);
        app.init_resource::<ElementInfoPluginConfig>();
        app.register_type::<ElementInfoPluginConfig>();
        app.register_type::<LatestTree>();
        app.register_type::<ElementInfo>();
        app.register_type::<AncestryTree>();
    }
}
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ElementInfoPluginConfig {
    pub refresh_interval: Timer,
}
impl Default for ElementInfoPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::from_seconds(0.01, TimerMode::Repeating),
        }
    }
}

#[derive(Default)]
pub struct WorkerState;

#[derive(Debug, Reflect, Clone, Event)]
pub enum ThreadboundMessage {
    Update,
}

fn handle_threadbound_message(
    msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    _state: &mut WorkerState,
    host_cursor_position: Res<HostCursorPosition>,
) -> Result<()> {
    let ThreadboundMessage::Update = msg;
    let tree = gather_tree_from_position(host_cursor_position.position)?;
    reply_tx.send(GameboundMessage { tree })?;
    Ok(())
}

#[derive(Debug, Reflect, Clone, Event)]
#[reflect(from_reflect = false)]
pub struct GameboundMessage {
    tree: AncestryTree,
}

#[derive(Component, Reflect)]
pub struct LatestTree;

fn handle_gamebound_messages(
    mut messages: EventReader<GameboundMessage>,
    mut commands: Commands,
    latest_tree: Query<Entity, With<LatestTree>>,
) -> Result {
    let Some(msg) = messages.read().last() else {
        return Ok(());
    };
    let GameboundMessage { tree } = msg;
    if let Ok(entity) = latest_tree.single() {
        commands.entity(entity).try_remove::<LatestTree>();
    }
    let time = chrono::Local::now();
    commands.spawn((
        tree.clone(),
        LatestTree,
        Name::new(format!("Tree - {time}")),
    ));

    Ok(())
}

fn tick_worker(
    mut threadbound_messages: EventWriter<ThreadboundMessage>,
    mut config: ResMut<ElementInfoPluginConfig>,
    time: Res<Time>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    threadbound_messages.write(ThreadboundMessage::Update);
}
