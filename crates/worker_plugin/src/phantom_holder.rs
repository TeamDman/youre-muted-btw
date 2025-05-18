use std::marker::PhantomData;

pub struct PhantomHolder<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
    ErrorFromMessageHandling,
    ErrorFromErrorHandling,
    ErrorFromMessageReceiving,
> {
    _phantom_t: PhantomData<ThreadboundMessage>,
    _phantom_g: PhantomData<GameboundMessage>,
    _phantom_s: PhantomData<WorkerState>,
    _phantom_e: PhantomData<ErrorFromMessageHandling>,
    _phantom_ee: PhantomData<ErrorFromErrorHandling>,
    _phantom_eee: PhantomData<ErrorFromMessageReceiving>,
}
unsafe impl<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
    ErrorFromMessageHandling,
    ErrorFromErrorHandling,
    ErrorFromMessageReceiving,
> Send
    for PhantomHolder<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
        ErrorFromErrorHandling,
        ErrorFromMessageReceiving,
    >
{
}
unsafe impl<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
    ErrorFromMessageHandling,
    ErrorFromErrorHandling,
    ErrorFromMessageReceiving,
> Sync
    for PhantomHolder<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
        ErrorFromErrorHandling,
        ErrorFromMessageReceiving,
    >
{
}
impl<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
    ErrorFromMessageHandling,
    ErrorFromErrorHandling,
    ErrorFromMessageReceiving,
> Clone
    for PhantomHolder<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
        ErrorFromErrorHandling,
        ErrorFromMessageReceiving,
    >
{
    fn clone(&self) -> Self {
        PhantomHolder {
            _phantom_t: PhantomData,
            _phantom_g: PhantomData,
            _phantom_s: PhantomData,
            _phantom_e: PhantomData,
            _phantom_ee: PhantomData,
            _phantom_eee: PhantomData,
        }
    }
}
impl<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
    ErrorFromMessageHandling,
    ErrorFromErrorHandling,
    ErrorFromMessageReceiving,
> Default
    for PhantomHolder<
        ThreadboundMessage,
        GameboundMessage,
        WorkerState,
        ErrorFromMessageHandling,
        ErrorFromErrorHandling,
        ErrorFromMessageReceiving,
    >
{
    fn default() -> Self {
        PhantomHolder {
            _phantom_t: PhantomData::<ThreadboundMessage>,
            _phantom_g: PhantomData::<GameboundMessage>,
            _phantom_s: PhantomData::<WorkerState>,
            _phantom_e: PhantomData::<ErrorFromMessageHandling>,
            _phantom_ee: PhantomData::<ErrorFromErrorHandling>,
            _phantom_eee: PhantomData::<ErrorFromMessageReceiving>,
        }
    }
}
