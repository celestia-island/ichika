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

pub trait IntoStatus<T, E> {
    fn into_status(self) -> Status<T, E>;
}

impl<T> IntoStatus<T, ()> for T {
    fn into_status(self) -> Status<T, ()> {
        Status::Next(self)
    }
}

impl<T, E> IntoStatus<T, E> for Result<T, E> {
    fn into_status(self) -> Status<T, E> {
        match self {
            Ok(value) => Status::Next(value),
            Err(error) => Status::Panic(error),
        }
    }
}

impl<T, E> IntoStatus<T, E> for Status<T, E> {
    fn into_status(self) -> Status<T, E> {
        self
    }
}
