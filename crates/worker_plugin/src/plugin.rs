use crate::WorkerConfig;
use crate::WorkerMessage;
use crate::WorkerStateTrait;
use crate::bridge_requests;
use crate::bridge_responses;
use crate::create_worker_thread;
use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::reflect::GetTypeRegistration;

pub struct WorkerPlugin<ThreadboundMessage, GameboundMessage, WorkerState>
where
    ThreadboundMessage: WorkerMessage,
    GameboundMessage: WorkerMessage,
    WorkerState: WorkerStateTrait,
{
    pub config: WorkerConfig<ThreadboundMessage, GameboundMessage, WorkerState>,
}

impl<ThreadboundMessage, GameboundMessage, WorkerState> Plugin
    for WorkerPlugin<ThreadboundMessage, GameboundMessage, WorkerState>
where
    ThreadboundMessage: WorkerMessage + GetTypeRegistration,
    GameboundMessage: WorkerMessage + GetTypeRegistration,
    WorkerState: WorkerStateTrait,
{
    fn build(&self, app: &mut App) {
        app.register_type::<ThreadboundMessage>();
        app.register_type::<GameboundMessage>();
        app.add_event::<ThreadboundMessage>();
        app.add_event::<GameboundMessage>();
        app.insert_resource(self.config.clone());
        app.add_systems(
            Startup,
            create_worker_thread::<ThreadboundMessage, GameboundMessage, WorkerState>,
        );
        app.add_systems(
            Update,
            bridge_requests::<ThreadboundMessage, GameboundMessage, WorkerState>,
        );
        app.add_systems(
            Update,
            bridge_responses::<ThreadboundMessage, GameboundMessage, WorkerState>,
        );
    }
}
