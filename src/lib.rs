//! hylic-pipeline — typestate pipelines and sugars over hylic lifts.
//!
//! Sits above `hylic` (core). Provides:
//!
//!   - Two Stage-1 pipeline typestates: [`SeedPipeline`], [`TreeishPipeline`]
//!   - Two Stage-2 pipeline typestates: [`LiftedPipeline`] (from TreeishPipeline;
//!     chain over `N`) and [`LiftedSeedPipeline`] (from SeedPipeline; chain over
//!     `SeedNode<N>` — see the Option-B design in
//!     `KB/.plans/project-entry-refactor/`).
//!   - One out-of-band one-shot pipeline: [`OwnedPipeline`]
//!   - Source interface traits: [`TreeishSource`], [`PipelineSourceOnce`]
//!   - Blanket execution traits: [`PipelineExec`], [`PipelineExecOnce`]
//!     ([`LiftedSeedPipeline`] has inherent `.run` / `.run_from_slice`;
//!     the seed axis is not a trait-level concern.)
//!   - Stage-1 sugar traits:
//!     - SeedPipeline: [`SeedSugarsShared`], [`SeedSugarsLocal`]
//!     - TreeishPipeline: [`TreeishSugarsShared`], [`TreeishSugarsLocal`]
//!   - Stage-2 sugars:
//!     - [`LiftedPipeline`] (seedless): [`LiftedSugarsShared`], [`LiftedSugarsLocal`].
//!     - [`LiftedSeedPipeline`]: inherent methods (`wrap_init`, `explain`, …).
//!       User closures are over `N`; Node/Entry dispatch is internal.
//!
//! Users who need only the lift-primitive surface (`Shared::wrap_init_lift`,
//! `Shared::n_lift`, `LiftBare::apply_bare`, …) can depend on `hylic`
//! alone. The pipeline layer adds typestate and chainable sugars.

#![warn(missing_docs)]

pub mod source;
pub mod seed;
pub mod treeish;
pub mod lifted;
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
pub use lifted::LiftedPipeline;
pub use lifted_seed::LiftedSeedPipeline;
pub use owned::OwnedPipeline;
pub use sugars::{
    SeedSugarsShared, SeedSugarsLocal,
    TreeishSugarsShared, TreeishSugarsLocal,
    LiftedSugarsShared, LiftedSugarsLocal,
};
pub use hylic::ops::SeedNode;
pub use stage2::{Wrap, Identity, SeedWrap, Stage2Base};
