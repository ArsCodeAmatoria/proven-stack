//! Application layer — the public interface (ADR-0006 §3) and its implementation. No HTTP, no
//! SQL: those live in `api` and `infrastructure` respectively.

pub mod apis;
pub mod ports;
pub mod services;

pub use apis::{UsersApi, UsersPorts, UsersServices};
