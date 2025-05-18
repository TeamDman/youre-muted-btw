use crate::PhantomHolder;
use crate::ThreadboundMessageErrorHandler;
use crate::ThreadboundMessageHandler;
use crate::ThreadboundMessageReceiver;
use crate::WorkerMessage;
use crate::WorkerStateTrait;
use bevy::ecs::resource::Resource;
use bevy::prelude::ReflectResource;
use bevy::reflect::Reflect;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct WorkerConfig<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
    ErrorFromMessageHandling,
    ErrorFromErrorHandling,
    ErrorFromMessageReceiving,
> {
    pub name: String,
    pub sleep_duration: std::time::Duration,
    pub is_ui_automation_thread: bool,
    pub threadbound_message_receiver:
        ThreadboundMessageReceiver<ThreadboundMessage, WorkerState, ErrorFromMessageReceiving>,
    pub handle_threadbound_message: ThreadboundMessageHandler<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
    >,
    pub handle_threadbound_message_error_handler: ThreadboundMessageErrorHandler<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
        ErrorFromErrorHandling,
    >,
    pub gamebound_channel_capacity: usize,
    pub threadbound_channel_capacity: usize,
    pub type_holder: PhantomHolder<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
        ErrorFromErrorHandling,
        ErrorFromMessageReceiving,
    >,
}
impl<ThreadboundMessage, GameboundMessage, WorkerState> Default
    for WorkerConfig<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        eyre::Error,
        eyre::Error,
        eyre::Error,
    >
where
    ThreadboundMessage: WorkerMessage,
    GameboundMessage: WorkerMessage,
    WorkerState: WorkerStateTrait,
{
    fn default() -> Self {
        WorkerConfig {
            name: "Unknown Worker".to_string(),
            is_ui_automation_thread: false,
            sleep_duration: std::time::Duration::ZERO,
            handle_threadbound_message: |_, _, _| Ok(()),
            handle_threadbound_message_error_handler: |_, _, _, _| Ok(()),
            threadbound_message_receiver: |thread_rx, _state| {
                thread_rx
                    .recv()
                    .map_err(|e| eyre::Error::from(e).wrap_err("receiving threadbound message"))
            },
            gamebound_channel_capacity: 10,
            threadbound_channel_capacity: 10,
            type_holder:
                PhantomHolder::<ThreadboundMessage, GameboundMessage, WorkerState, _, _, _>::default(
                ),
        }
    }
}
impl<T, G, S, E, EE, EEE> Clone for WorkerConfig<T, G, S, E, EE, EEE> {
    fn clone(&self) -> Self {
        WorkerConfig {
            name: self.name.clone(),
            sleep_duration: self.sleep_duration,
            is_ui_automation_thread: self.is_ui_automation_thread,
            threadbound_message_receiver: self.threadbound_message_receiver,
            handle_threadbound_message: self.handle_threadbound_message,
            handle_threadbound_message_error_handler: self.handle_threadbound_message_error_handler,
            gamebound_channel_capacity: self.gamebound_channel_capacity,
            threadbound_channel_capacity: self.threadbound_channel_capacity,
            type_holder: self.type_holder.clone(),
        }
    }
}
