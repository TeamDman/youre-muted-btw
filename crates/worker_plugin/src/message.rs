use bevy::ecs::event::Event;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

pub trait WorkerMessage: std::fmt::Debug + Event + Send + Sync + Clone + 'static {}
impl<T> WorkerMessage for T where T: std::fmt::Debug + Event + Send + Sync + Clone + 'static {}

pub type ThreadboundMessageHandler<T, G, S, E> =
    fn(msg: &T, reply_tx: &Sender<G>, state: &mut S) -> Result<(), E>;

pub type ThreadboundMessageErrorHandler<T, G, S, ErrorFromMsgHandling, ErrorFromErrorHandling> =
    fn(
        msg: &T,
        reply_tx: &Sender<G>,
        state: &mut S,
        error: &ErrorFromMsgHandling,
    ) -> Result<(), ErrorFromErrorHandling>;

pub type ThreadboundMessageReceiver<T, S, E> =
    fn(thread_rx: &Receiver<T>, state: &mut S) -> Result<T, E>;
