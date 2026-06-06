//! Business Flow Designer — Tauri backend module
//!
//! Provides visual BPMN flow editing, validation, and execution for
//! multi-agent (硅基军团) business workflows.
//!
//! ## Architecture
//! ```text
//! Frontend (Vue Flow) ──IPC──▶ handler.rs (commands)
//!                                  ├── model.rs  (data types + DB access)
//!                                  ├── conductor.rs (DAG runtime)
//!                                  └── validation.rs (static checks)
//! ```
//!
//! ## Tables
//! - `business_flows` — flow definitions + JSON graph
//! - `flow_runs` — execution tracking
//! - `flow_run_artifacts` — per-node output files

pub mod handler;
pub mod model;
pub mod validation;
pub mod conductor;
