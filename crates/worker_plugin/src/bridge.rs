use crate::WorkerConfig;
use crate::WorkerMessage;
use crate::WorkerStateTrait;
use bevy::ecs::event::EventReader;
use bevy::ecs::event::EventWriter;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::log::error;
use bevy::log::trace;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

pub fn bridge_requests<ThreadboundMessage, GameboundMessage, WorkerState>(
    config: Res<WorkerConfig<ThreadboundMessage, GameboundMessage, WorkerState>>,
    bridge: ResMut<Bridge<ThreadboundMessage, GameboundMessage>>,
    mut events: EventReader<ThreadboundMessage>,
) where
    ThreadboundMessage: WorkerMessage,
    GameboundMessage: WorkerMessage,
    WorkerState: WorkerStateTrait,
{
    for event in events.read() {
        trace!("[{}] Bevy => Thread: {:?}", config.name, event);
        if let Err(e) = bridge.sender.try_send(event.clone()) {
            match e {
                crossbeam_channel::TrySendError::Full(_) => {
                    error!(
                        "[{}] Threadbound channel is full, dropping message: {:?}",
                        config.name, event
                    );
                }
                crossbeam_channel::TrySendError::Disconnected(_) => {
                    error!(
                        "[{}] Threadbound channel is disconnected, dropping message: {:?}",
                        config.name, event
                    );
                }
            }
        }
    }
}

pub fn bridge_responses<T, G, S>(
    config: Res<WorkerConfig<T, G, S>>,
    bridge: ResMut<Bridge<T, G>>,
    mut events: EventWriter<G>,
) where
    T: WorkerMessage,
    G: WorkerMessage,
    S: WorkerStateTrait,
{
    for msg in bridge.receiver.try_iter() {
        trace!("[{}] Thread => Bevy: {:?}", config.name, msg);
        events.write(msg);
    }
}

#[derive(Resource)]
pub struct Bridge<T, G>
where
    T: WorkerMessage,
    G: WorkerMessage,
{
    pub sender: Sender<T>,
    pub receiver: Receiver<G>,
}
