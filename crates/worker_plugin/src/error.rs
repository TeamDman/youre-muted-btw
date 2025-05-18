pub trait WorkerError: std::fmt::Debug + 'static {}
impl<T> WorkerError for T where T: std::fmt::Debug + 'static {}
