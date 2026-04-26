//! Stage-2 pipeline machinery — unified across treeish-rooted and
//! seed-rooted bases.
//!
//! - [`Stage2Pipeline<Base, L>`] is the unified Stage-2 typestate.
//! - [`Wrap`] is the type-level dispatch trait for chain N wrapping.
//! - [`Stage2Base`] connects Stage-1 bases to their `Wrap`.
//!
//! The two old types (`LiftedPipeline`, `LiftedSeedPipeline`) are
//! deprecated aliases of `Stage2Pipeline`, kept for one cycle.
//!
//! Stage-2 sugars currently live in `crate::sugars::lifted_*` and
//! `crate::lifted_seed::sugars_*` — these target
//! `LiftedPipeline`/`LiftedSeedPipeline` (= Stage2Pipeline aliases)
//! directly. Phase 4 of the seed-pipeline-unification plan will fold
//! them into a single Wrap-dispatched surface; for now they remain
//! per-(Base × Domain) inherent and trait impls.

pub mod wrap;
pub mod base;
pub mod pipeline;
pub mod primitives;
pub mod source_impl;
pub mod run_seed_shared;
pub mod run_seed_local;

pub use wrap::{Wrap, Identity, SeedWrap};
pub use base::Stage2Base;
pub use pipeline::Stage2Pipeline;
