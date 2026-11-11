//! Model auditing for SeaORM, inspired by `owen-it/laravel-auditing`.
//!
//! * [`entity`] / [`migration`] – the `audits` table.
//! * [`Auditable`] – implement on your entity `Model` to configure auditing.
//! * [`Auditor`] – `insert` / `update` / `delete` wrappers that record audits.
//! * [`AuditContext`] – request data (user, url, ip, ...) carried in a task-local,
//!   set by the Salvo (`salvo` feature) or Loco/axum (`loco` feature) middleware.

pub mod auditable;
pub mod auditor;
pub mod context;
pub mod entity;
pub mod error;
pub mod migration;

#[cfg(feature = "loco")]
pub mod loco;
#[cfg(feature = "salvo")]
pub mod salvo;

pub use auditable::{AuditData, AuditEvent, Auditable};
pub use auditor::Auditor;
pub use context::AuditContext;
pub use error::AuditError;
