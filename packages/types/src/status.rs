#[derive(Clone)]
pub enum Status<T, E> {
    Next(T),
    Switch((&'static str, T)),
    Panic(E),
    PanicSwitch((&'static str, E)),
    Back((&'static str, T)),
    Retry,
    Exit,
}

impl<T> From<T> for Status<T, ()> {
    fn from(value: T) -> Self {
        Status::Next(value)
    }
}

impl<T, E> From<Result<T, E>> for Status<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Status::Next(value),
            Err(error) => Status::Panic(error),
        }
    }
}
