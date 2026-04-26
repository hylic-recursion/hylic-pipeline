//! hylic-pipeline — typestate pipelines and sugars over hylic lifts.
//!
//! Sits above `hylic` (core). Provides:
//!
//!   - Two Stage-1 pipeline typestates: [`SeedPipeline`], [`TreeishPipeline`]
//!   - One unified Stage-2 pipeline typestate: [`Stage2Pipeline<Base, L>`].
//!     Distinguished only by the wrapped Stage-1 base (`TreeishPipeline`
//!     or `SeedPipeline`) and which `.run` is callable. The chain's
//!     input N is `<Base::Wrap as Wrap>::Of<UserN>` — `UserN` for
//!     treeish-rooted (`Identity` wrap), `SeedNode<UserN>` for
//!     seed-rooted (`SeedWrap`).
//!   - Backward-compat aliases: `LiftedPipeline` and `LiftedSeedPipeline`
//!     are deprecated aliases of `Stage2Pipeline`.
//!   - One out-of-band one-shot pipeline: [`OwnedPipeline`]
//!   - Source interface traits: [`TreeishSource`], [`PipelineSourceOnce`]
//!   - Blanket execution traits: [`PipelineExec`], [`PipelineExecOnce`]
//!   - Stage-1 sugar traits:
//!     - SeedPipeline: [`SeedSugarsShared`], [`SeedSugarsLocal`]
//!     - TreeishPipeline: [`TreeishSugarsShared`], [`TreeishSugarsLocal`]
//!   - Stage-2 sugars: trait-based on the treeish-rooted side
//!     (`LiftedSugarsShared`/`Local`) and inherent on the seed-rooted
//!     side. Phase 4 of the seed-pipeline-unification plan will
//!     unify these.
//!
//! Users who need only the lift-primitive surface (`Shared::wrap_init_lift`,
//! `Shared::n_lift`, `LiftBare::apply_bare`, …) can depend on `hylic`
//! alone.

#![warn(missing_docs)]

pub mod source;
pub mod seed;
pub mod treeish;
#[allow(deprecated)]
pub mod lifted;
#[allow(deprecated)]
pub mod lifted_seed;
pub mod owned;
pub mod sugars;
pub mod stage2;

#[cfg(test)]
mod tests;

pub mod prelude;

pub use source::{
    TreeishSource,
    PipelineSourceOnce,
    PipelineExec, PipelineExecOnce,
};
pub use seed::SeedPipeline;
pub use treeish::TreeishPipeline;
#[allow(deprecated)]
pub use lifted::LiftedPipeline;
#[allow(deprecated)]
pub use lifted_seed::LiftedSeedPipeline;
pub use owned::OwnedPipeline;
pub use sugars::{
    SeedSugarsShared, SeedSugarsLocal,
    TreeishSugarsShared, TreeishSugarsLocal,
    LiftedSugarsShared, LiftedSugarsLocal,
};
pub use hylic::ops::SeedNode;
pub use stage2::{Wrap, Identity, SeedWrap, Stage2Base, Stage2Pipeline};
