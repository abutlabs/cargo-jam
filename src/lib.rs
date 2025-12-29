//! cargo-jam: Generate JAM service projects for Polkadot
//!
//! This crate provides a cargo subcommand for generating JAM (Join-Accumulate Machine)
//! service projects for Polkadot. It follows the architecture of cargo-generate while
//! providing JAM-specific templates and build tooling.
//!
//! ## Usage
//!
//! ```bash
//! # Create a new JAM service
//! cargo jam new my-service
//!
//! # Build a JAM service for PVM deployment
//! cargo jam build --release
//! ```

pub mod build;
pub mod cli;
pub mod error;
pub mod project;
pub mod prompt;
pub mod template;

pub use error::{CargoJamError, Result};
