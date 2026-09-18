//! `agent-rs` is a template for Rust command-line projects built by people and coding agents.
//!
//! `main.rs` only wires the process. [`cli`] owns the command-line contract; every other module
//! takes typed input and returns typed values and errors to it.

pub mod build_info;
pub mod cli;
