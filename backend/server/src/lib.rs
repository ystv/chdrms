pub mod auth;
pub mod config;
pub mod error;
pub mod routes;
pub mod state;

// TODO: include instance information in UA
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
