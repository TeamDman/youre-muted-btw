use bevy::prelude::*;
use ymb_ui_automation::gather_elements_at;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

pub struct DiscordAppWorkerPlugin;

impl Plugin for DiscordAppWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<ThreadboundMessage, GameboundMessage, ()> {
                name: "DiscordAppWorker".to_string(),
                handle_threadbound_message,
                ..default()
            },
        });
        app.add_systems(Update, handle_gamebound_messages);
    }
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum ThreadboundMessage {
    Lookup { pos: IVec2 },
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum GameboundMessage {
    Found { drill_id: String },
}

fn handle_threadbound_message(
    msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    _state: &mut (),
) -> Result<()> {
    match msg {
        ThreadboundMessage::Lookup { pos } => {
            info!("Gathering elements at position: {:?}", pos);
            let elements = gather_elements_at(*pos)?;
            for (element, depth) in elements {
                info!("Found element: {depth} {element:?}");
                if element.get_name() == Ok("Mute".to_string()) {
                    reply_tx.send(GameboundMessage::Found {
                        drill_id: format!("{:?}", element),
                    })?;
                }
            }
        }
    }
    Ok(())
}

fn handle_gamebound_messages(mut messages: EventReader<GameboundMessage>) {
    for msg in messages.read() {
        match msg {
            GameboundMessage::Found { drill_id } => {
                info!(
                    "Found Discord mute button UI element with DrillID: {}",
                    drill_id
                );
            }
        }
    }
}
