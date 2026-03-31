/// Retry metadata for controlling retry behavior.
#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retry, 1 = one retry, etc.)
    pub max_attempts: usize,
    /// Delay between retry attempts in milliseconds
    pub delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay_ms: 100,
        }
    }
}

#[derive(Clone)]
pub enum Status<T, E> {
    Next(T),
    Switch((&'static str, T)),
    Panic(E),
    PanicSwitch((&'static str, E)),
    Back((&'static str, T)),
    /// Retry with policy metadata and current attempt count (0-indexed)
    RetryWith(RetryPolicy, usize, T),
    /// Immediate retry (legacy, uses default policy)
    Retry,
    Exit,
}

/// Helper function to create a Status::Retry with explicit type parameters.
/// Use this when the compiler cannot infer the type parameters from context.
pub fn retry<T, E>() -> Status<T, E> {
    Status::Retry
}

/// Helper function to create a Status::RetryWith with explicit type parameters.
pub fn retry_with<T: Clone>(policy: RetryPolicy, attempt: usize, value: T) -> Status<T, ()> {
    Status::RetryWith(policy, attempt, value)
}

pub trait IntoStatus<T, E> {
    fn into_status(self) -> Status<T, E>;
}

impl<T, E> IntoStatus<T, E> for Result<T, E> {
    fn into_status(self) -> Status<T, E> {
        match self {
            Ok(value) => Status::Next(value),
            Err(error) => Status::Panic(error),
        }
    }
}

// Identity implementation for Status<T, E> -> Status<T, E>
impl<T, E> IntoStatus<T, E> for Status<T, E> {
    fn into_status(self) -> Status<T, E> {
        self
    }
}
