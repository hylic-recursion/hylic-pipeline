//! LiftedSeedPipeline — Stage 2 of the SeedPipeline lifecycle.
//!
//! Distinct from `LiftedPipeline` because its chain is typed at
//! `SeedNode<N>` rather than `N`. At `.run` time, `SeedLift` is
//! assembled from the `SeedPipeline`'s `grow` and user-supplied
//! `root_seeds` + `entry_heap`, and composed as the first lift in
//! the chain. Everything above it operates on `SeedNode<N>` —
//! `Entry` is a first-class value of the node type.

use hylic::ops::IdentityLift;

pub mod primitives;
pub mod run;
pub mod run_local;
pub mod sugars_shared;
pub mod sugars_local;
pub(crate) mod gat_helpers;

// ANCHOR: lifted_seed_pipeline_struct
/// Stage-2 typestate pipeline rooted at a `SeedPipeline`. Wraps the
/// base with a lift chain `L` typed at `SeedNode<N>`. `SeedLift` is
/// NOT yet constructed — it's assembled at `.run` time when the user
/// supplies the root seeds and entry heap.
#[must_use]
pub struct LiftedSeedPipeline<Base, L = IdentityLift> {
    pub(crate) base:     Base,
    pub(crate) pre_lift: L,
}
// ANCHOR_END: lifted_seed_pipeline_struct

impl<Base, L> LiftedSeedPipeline<Base, L> {
    pub(crate) fn new(base: Base, pre_lift: L) -> Self {
        LiftedSeedPipeline { base, pre_lift }
    }
}

impl<Base: Clone, L: Clone> Clone for LiftedSeedPipeline<Base, L> {
    fn clone(&self) -> Self {
        LiftedSeedPipeline {
            base:     self.base.clone(),
            pre_lift: self.pre_lift.clone(),
        }
    }
}
