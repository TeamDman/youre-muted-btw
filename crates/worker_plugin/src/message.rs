use bevy::ecs::event::Event;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

pub trait WorkerMessage: std::fmt::Debug + Event + Send + Sync + Clone + 'static {}
impl<T> WorkerMessage for T where T: std::fmt::Debug + Event + Send + Sync + Clone + 'static {}

pub type ThreadboundMessageHandler<ThreadboundMessage, GameboundMessage, WorkerState> =
    fn(
        msg: &ThreadboundMessage,
        reply_tx: &Sender<GameboundMessage>,
        state: &mut WorkerState,
    ) -> bevy::prelude::Result<()>;

pub type ThreadboundMessageErrorHandler<ThreadboundMessage, GameboundMessage, WorkerState> =
    fn(
        msg: &ThreadboundMessage,
        reply_tx: &Sender<GameboundMessage>,
        state: &mut WorkerState,
        error: &bevy::prelude::BevyError,
    ) -> bevy::prelude::Result<()>;

pub type ThreadboundMessageReceiver<T, S> =
    fn(thread_rx: &Receiver<T>, state: &mut S) -> bevy::prelude::Result<T>;
