pub mod agent;
pub mod events;
pub mod firecracker;
pub mod fstack;
pub mod instance;
pub mod jailer;
pub mod models;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Agent: {0}")]
    Agent(String),
    #[error("Configuraion: {0}")]
    Configuration(String),
    #[error("Event: {0}")]
    Event(String),
    #[error("Instance: {0}")]
    Instance(String),
    #[error("{0}")]
    FeatureNone(String),
}

pub type Result<T> = std::result::Result<T, crate::Error>;

#[macro_export]
macro_rules! missing_rt {
    () => {
        Err(crate::Error::FeatureNone(
            r#"Must enable one of these features: "_rt-std", "_rt-tokio", "_rt-async-std""#.into(),
        ))
    };
}

#[macro_export]
macro_rules! missing_rt_panic {
    () => {
        panic!(r#"Must enable one of these features: "_rt-std", "_rt-tokio", "_rt-async-std""#)
    };
}
