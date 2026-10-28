use crate::domain::model::{ProcessInfo, Snapshot};
pub type AppResult<T> = Result<T, String>;
pub trait PortScanner: Send + Sync {
    fn scan(&self) -> AppResult<Snapshot>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Terminate,
    Kill,
}
pub trait ProcessControl: Send + Sync {
    fn inspect(&self, pid: u32) -> AppResult<Option<ProcessInfo>>;
    fn signal(&self, pid: u32, signal: Signal) -> AppResult<()>;
}
pub trait Clock: Send + Sync {
    fn now(&self) -> u64;
    fn sleep(&self, millis: u64);
}
