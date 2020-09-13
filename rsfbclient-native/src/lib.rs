//! `FirebirdConnection` implementation for the native fbclient

mod connection;
pub(crate) mod ibase;
pub(crate) mod params;
pub(crate) mod row;
pub(crate) mod status;
pub(crate) mod xsqlda;
pub(crate) mod varchar;

pub use connection::{Args, NativeFbClient};
