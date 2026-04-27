//! Stage-2 pipeline machinery — unified across treeish-rooted and
//! seed-rooted bases.
//!
//! - [`Stage2Pipeline<Base, L>`] is the unified Stage-2 typestate.
//! - [`Wrap`] is the type-level dispatch trait for chain N wrapping.
//! - [`WrapShared`] / [`WrapLocal`] add per-domain build methods used
//!   by the unified Stage-2 sugar surface.
//! - [`Stage2Base`] connects Stage-1 bases to their `Wrap`.
//!
//! `aliases.rs` retains the deprecated `LiftedPipeline` /
//! `LiftedSeedPipeline` type aliases for one cycle.

pub mod wrap;
pub mod wrap_shared;
pub mod wrap_local;
pub mod base;
pub mod pipeline;
pub mod primitives;
pub mod source_impl;
pub mod run_seed_shared;
pub mod run_seed_local;
pub mod aliases;
pub(crate) mod gat_helpers;

pub use wrap::{Wrap, Identity, SeedWrap};
pub use wrap_shared::WrapShared;
pub use wrap_local::WrapLocal;
pub use base::Stage2Base;
pub use pipeline::Stage2Pipeline;
#[allow(deprecated)]
pub use aliases::{LiftedPipeline, LiftedSeedPipeline};
