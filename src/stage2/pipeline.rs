//! `Stage2Pipeline<Base, L>` — the unified Stage-2 typestate.
//!
//! Wraps a Stage-1 `Base` source with a single lift `L` (usually a
//! `ComposedLift` tree built from sugar chaining). Replaces both the
//! old `Stage2Pipeline` (TreeishPipeline-rooted) and `Stage2Pipeline`
//! (SeedPipeline-rooted) with a single struct distinguished only by
//! which `Base` it wraps and which `.run` is callable.
//!
//! The chain's input N is `<Base::Wrap as Wrap>::Of<UN>` —
//! `UN` for treeish-rooted bases (`Identity` wrap), `SeedNode<UN>` for
//! seed-rooted bases (`SeedWrap`). Sugars dispatch user closures over
//! `&UN` via `Base::Wrap::project`.

use hylic::ops::IdentityLift;

// ANCHOR: stage2_pipeline_struct
/// Stage-2 typestate pipeline. Wraps a Stage-1 base with a lift chain.
/// The chain's input N is `<Base::Wrap as Wrap>::Of<UN>` — see the
/// `Stage2Base` and `Wrap` traits in this module.
#[must_use]
pub struct Stage2Pipeline<Base, L = IdentityLift> {
    pub(crate) base:     Base,
    pub(crate) pre_lift: L,
}
// ANCHOR_END: stage2_pipeline_struct

impl<Base, L> Stage2Pipeline<Base, L> {
    pub(crate) fn new(base: Base, pre_lift: L) -> Self {
        Stage2Pipeline { base, pre_lift }
    }
}

impl<Base: Clone, L: Clone> Clone for Stage2Pipeline<Base, L> {
    fn clone(&self) -> Self {
        Stage2Pipeline {
            base:     self.base.clone(),
            pre_lift: self.pre_lift.clone(),
        }
    }
}
