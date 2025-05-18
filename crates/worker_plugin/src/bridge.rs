use crate::WorkerConfig;
use crate::WorkerError;
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

pub fn bridge_requests<T, G, S, E, EE, EEE>(
    config: Res<WorkerConfig<T, G, S, E, EE, EEE>>,
    bridge: ResMut<Bridge<T, G>>,
    mut events: EventReader<T>,
) where
    T: WorkerMessage,
    G: WorkerMessage,
    S: WorkerStateTrait,
    E: WorkerError,
    EE: WorkerError,
    EEE: WorkerError,
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

pub fn bridge_responses<T, G, S, E, EE, EEE>(
    config: Res<WorkerConfig<T, G, S, E, EE, EEE>>,
    bridge: ResMut<Bridge<T, G>>,
    mut events: EventWriter<G>,
) where
    T: WorkerMessage,
    G: WorkerMessage,
    S: WorkerStateTrait,
    E: WorkerError,
    EE: WorkerError,
    EEE: WorkerError,
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
