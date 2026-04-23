//! LiftedPipeline — Stage 2 of the pipeline typestate.
//!
//! Parametric over its source: `LiftedPipeline<Base, L>` wraps any
//! `Base: PipelineSource` (SeedPipeline, TreeishPipeline, or any
//! future source) with a lift chain `L`. Sole Stage-2 primitive:
//! `then_lift`.

use hylic::ops::IdentityLift;

pub mod primitives;
pub mod source_impl;

// ANCHOR: lifted_pipeline_struct
/// Stage-2 typestate pipeline. Wraps a Stage-1 `Base` source with
/// a single lift `L` (usually a `ComposedLift` tree built from
/// sugar chaining). Implements both `TreeishSource` and, where
/// `Base` does, `SeedSource`.
#[must_use = "a LiftedPipeline carries the transformation plan; call `.run_from_node(...)` to execute it"]
pub struct LiftedPipeline<Base, L = IdentityLift> {
    pub(crate) base:     Base,
    pub(crate) pre_lift: L,
}
// ANCHOR_END: lifted_pipeline_struct

impl<Base, L> LiftedPipeline<Base, L> {
    pub(crate) fn new(base: Base, pre_lift: L) -> Self {
        LiftedPipeline { base, pre_lift }
    }
}

impl<Base: Clone, L: Clone> Clone for LiftedPipeline<Base, L> {
    fn clone(&self) -> Self {
        LiftedPipeline {
            base:     self.base.clone(),
            pre_lift: self.pre_lift.clone(),
        }
    }
}
