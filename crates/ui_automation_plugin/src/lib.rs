use bevy::prelude::*;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use std::time::Duration;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_ui_automation::AncestryTree;
use ymb_ui_automation::ElementInfo;
use ymb_ui_automation::YMBControlType;
use ymb_ui_automation::gather_root;
use ymb_ui_automation::gather_tree_from_position;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

pub struct ElementInfoPlugin;

impl Plugin for ElementInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<
                UIWorkerThreadboundMessage,
                UIWorkerGameboundMessage,
                UIWorkerState,
            > {
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
        app.add_systems(Startup, startup_fetch);
        app.init_resource::<ElementInfoPluginConfig>();
        app.register_type::<ElementInfoPluginConfig>();
        app.register_type::<LatestInfo>();
        app.register_type::<RevealIntent>();
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
            refresh_interval: Timer::new(
                Duration::MAX, // disable for now
                TimerMode::Repeating,
            ),
        }
    }
}

#[derive(Default)]
pub struct UIWorkerState;

#[derive(Debug, Reflect, Clone, Event)]
pub enum UIWorkerThreadboundMessage {
    UpdateFromPos { host_cursor_position: IVec2 },
    UpdateRoot,
}

fn handle_threadbound_message(
    msg: &UIWorkerThreadboundMessage,
    reply_tx: &Sender<UIWorkerGameboundMessage>,
    _state: &mut UIWorkerState,
) -> Result<()> {
    info!("Handling threadbound message: {:?}", msg);
    match msg {
        UIWorkerThreadboundMessage::UpdateFromPos {
            host_cursor_position,
        } => {
            let tree = gather_tree_from_position(*host_cursor_position)?;
            reply_tx.send(UIWorkerGameboundMessage::UIFromPos { tree })?;
        }
        UIWorkerThreadboundMessage::UpdateRoot => {
            let root = gather_root()?;
            reply_tx.send(UIWorkerGameboundMessage::UIFromRoot { root })?;
        }
    }
    info!("Threadbound message handled successfully.");
    Ok(())
}

#[derive(Debug, Reflect, Clone, Event)]
#[reflect(from_reflect = false)]
pub enum UIWorkerGameboundMessage {
    UIFromPos { tree: AncestryTree },
    UIFromRoot { root: ElementInfo },
}

#[derive(Component, Reflect)]
pub struct LatestInfo;

#[derive(Component, Reflect)]
pub struct RevealIntent {
    pub element_info: ElementInfo,
}

fn handle_gamebound_messages(
    mut messages: EventReader<UIWorkerGameboundMessage>,
    mut commands: Commands,
    latest_tree: Query<Entity, With<LatestInfo>>,
) -> Result {
    let Some(msg) = messages.read().last() else {
        return Ok(());
    };
    match msg {
        UIWorkerGameboundMessage::UIFromPos { tree } => {
            let time = chrono::Local::now();
            commands.spawn((
                tree.tree.clone(),
                LatestInfo,
                RevealIntent {
                    element_info: tree.start.clone(),
                },
                Name::new(format!("Tree - {time}")),
            ));
        }
        UIWorkerGameboundMessage::UIFromRoot { root } => {
            let time = chrono::Local::now();
            commands.spawn((
                root.clone(),
                LatestInfo,
                Name::new(format!("Tree - {time}")),
            ));
        }
    }

    // Remove the previous latest element if it exists
    if let Ok(entity) = latest_tree.single() {
        commands.entity(entity).try_remove::<LatestInfo>();
    }

    Ok(())
}

fn tick_worker(
    mut threadbound_messages: EventWriter<UIWorkerThreadboundMessage>,
    mut config: ResMut<ElementInfoPluginConfig>,
    time: Res<Time>,
    host_cursor_position: Res<HostCursorPosition>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    threadbound_messages.write(UIWorkerThreadboundMessage::UpdateFromPos {
        host_cursor_position: host_cursor_position.position,
    });
}

fn startup_fetch(mut threadbound_messages: EventWriter<UIWorkerThreadboundMessage>) {
    threadbound_messages.write(UIWorkerThreadboundMessage::UpdateRoot);
}
