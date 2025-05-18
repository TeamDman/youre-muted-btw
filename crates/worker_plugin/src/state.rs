pub trait WorkerStateTrait: 'static + Sized {
    type Error;
    fn try_default() -> Result<Self, Self::Error>;
}
impl<T> WorkerStateTrait for T
where
    T: Default + 'static,
{
    type Error = ();
    fn try_default() -> Result<Self, Self::Error> {
        Ok(T::default())
    }
}
