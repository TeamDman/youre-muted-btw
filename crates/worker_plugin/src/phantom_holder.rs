use std::marker::PhantomData;

pub struct PhantomHolder<ThreadboundMessage, GameboundMessage, WorkerState> {
    _phantom_t: PhantomData<ThreadboundMessage>,
    _phantom_g: PhantomData<GameboundMessage>,
    _phantom_s: PhantomData<WorkerState>,
}
unsafe impl<ThreadboundMessage, GameboundMessage, WorkerState> Send
    for PhantomHolder<ThreadboundMessage, GameboundMessage, WorkerState>
{
}
unsafe impl<ThreadboundMessage, GameboundMessage, WorkerState> Sync
    for PhantomHolder<ThreadboundMessage, GameboundMessage, WorkerState>
{
}
impl<ThreadboundMessage, GameboundMessage, WorkerState> Clone
    for PhantomHolder<ThreadboundMessage, GameboundMessage, WorkerState>
{
    fn clone(&self) -> Self {
        PhantomHolder {
            _phantom_t: PhantomData,
            _phantom_g: PhantomData,
            _phantom_s: PhantomData,
        }
    }
}
impl<ThreadboundMessage, GameboundMessage, WorkerState> Default
    for PhantomHolder<ThreadboundMessage, GameboundMessage, WorkerState>
{
    fn default() -> Self {
        PhantomHolder {
            _phantom_t: PhantomData::<ThreadboundMessage>,
            _phantom_g: PhantomData::<GameboundMessage>,
            _phantom_s: PhantomData::<WorkerState>,
        }
    }
}
