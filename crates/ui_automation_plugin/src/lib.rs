use bevy::prelude::*;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_ui_automation::AncestryTree;
use ymb_ui_automation::ElementInfo;
use ymb_ui_automation::gather_tree_from_position;
use ymb_ui_automation::YMBControlType;
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
                threadbound_message_receiver: |thread_rx, _state| {
                    if thread_rx.is_empty() {
                        thread_rx.recv().map_err(BevyError::from)
                    } else {
                        Ok(thread_rx.try_iter().last().unwrap())
                    }
                },
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
        app.register_type::<YMBControlType>();
        app.register_type_data::<YMBControlType, InspectorEguiImpl>();
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
            refresh_interval: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

#[derive(Default)]
pub struct WorkerState;

#[derive(Debug, Reflect, Clone, Event)]
pub enum ThreadboundMessage {
    Update { host_cursor_position: IVec2 },
}

fn handle_threadbound_message(
    msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    _state: &mut WorkerState,
) -> Result<()> {
    let ThreadboundMessage::Update {
        host_cursor_position,
    } = msg;
    info!("ThreadboundMessage::Update: {host_cursor_position:?}");
    let tree = gather_tree_from_position(*host_cursor_position)?;
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
    host_cursor_position: Res<HostCursorPosition>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    threadbound_messages.write(ThreadboundMessage::Update {
        host_cursor_position: host_cursor_position.position,
    });
}
