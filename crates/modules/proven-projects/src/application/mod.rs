//! Application layer — public `ProjectsApi` and services (ADR-0009).

pub mod apis;
pub mod ports;
pub mod services;

pub use apis::{ProjectsApi, ProjectsPorts, ProjectsServices};
