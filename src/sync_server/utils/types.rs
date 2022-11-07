use std::sync::{mpsc::{RecvError, Receiver, SendError}, PoisonError, MutexGuard};

#[derive(Debug)]
pub enum ServerError {
    StreanReadingError(String),
    LockError(String),
    BadRequest(String),
    ExecutingError(String),
}
impl From<RecvError> for ServerError {
    fn from(value: RecvError) -> Self {
        Self::LockError(
            format!("failed to receive a value: {}", value.to_string())
        )
    }
}
impl From<PoisonError<MutexGuard<'_, Receiver<Message>>>> for ServerError {
    fn from(value: PoisonError<MutexGuard<Receiver<Message>>>) -> Self {
        Self::LockError(
            format!("failed to take a MutexGuard of reciever: {}",
            value.to_string()
        ))
    }
}
impl From<SendError<Message>> for ServerError {
    fn from(value: SendError<Message>) -> Self {
        Self::ExecutingError(
            format!("faield to execute a job: {}",
            value.to_string()
        ))
    }
}
impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        Self::StreanReadingError(
            format!("failed to read TcpStream: {}",
            value.to_string()
        ))
    }
}

pub(crate) type Job = Box<dyn FnOnce() + Send + 'static>;
pub(crate) enum Message {
    NewJob(Job),
    Terminate,
}