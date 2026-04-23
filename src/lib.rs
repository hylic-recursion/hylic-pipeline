//! hylic-pipeline — typestate pipelines and sugars over hylic lifts.
//!
//! Sits above `hylic` (core). Provides:
//!
//!   - Two Stage-1 pipeline typestates: [`SeedPipeline`], [`TreeishPipeline`]
//!   - One Stage-2 pipeline typestate: [`LiftedPipeline`]
//!   - One out-of-band one-shot pipeline: [`OwnedPipeline`]
//!   - Source interface traits: [`TreeishSource`], [`SeedSource`]
//!   - Blanket execution traits: [`PipelineExec`], [`PipelineExecSeed`], [`PipelineExecOnce`]
//!   - Blanket sugar traits:
//!     - Stage 1 (SeedPipeline): [`SeedSugarsShared`], [`SeedSugarsLocal`]
//!       (`filter_seeds`, `wrap_grow`, `map_node_bi`, `map_seed_bi`).
//!     - Stage 1 (TreeishPipeline): [`TreeishSugarsShared`], [`TreeishSugarsLocal`]
//!       (just `map_node_bi`).
//!     - Stage 2: [`LiftedSugarsShared`], [`LiftedSugarsLocal`].
//!       On `LiftedPipeline` use `.then_lift(Shared::map_n_bi_lift(co, contra))`
//!       for N-change (no auto-lift method to avoid overlap with Stage-1).
//!
//! Users who need only the lift-primitive surface (`Shared::wrap_init_lift`,
//! `Shared::n_lift`, `LiftBare::apply_bare`, …) can depend on `hylic`
//! alone. The pipeline layer adds typestate and chainable sugars.

pub mod source;
pub mod seed;
pub mod treeish;
pub mod lifted;
pub mod owned;
pub mod sugars;

#[cfg(test)]
mod tests;

pub mod prelude;

pub use source::{
    TreeishSource, SeedSource,
    PipelineSourceOnce,
    PipelineExec, PipelineExecSeed, PipelineExecOnce,
};
pub use seed::SeedPipeline;
pub use treeish::TreeishPipeline;
pub use lifted::LiftedPipeline;
pub use owned::OwnedPipeline;
pub use sugars::{
    SeedSugarsShared, SeedSugarsLocal,
    TreeishSugarsShared, TreeishSugarsLocal,
    LiftedSugarsShared, LiftedSugarsLocal,
};
pub use hylic::ops::LiftedNode;
