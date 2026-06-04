//! HTTP client for the Nubster control-plane API.

pub mod client;
pub mod repo;

#[allow(unused_imports)] // Consumed once commands are wired in #8-#9; drop this allow there.
pub use client::{Client, RetryPolicy};
