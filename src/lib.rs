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
//!   - One out-of-band one-shot pipeline: [`OwnedPipeline`]
//!   - Source interface traits: [`TreeishSource`], [`PipelineSourceOnce`]
//!   - Blanket execution traits: [`PipelineExec`], [`PipelineExecOnce`]
//!   - Stage-1 sugar traits:
//!     - SeedPipeline: [`SeedSugarsShared`], [`SeedSugarsLocal`]
//!     - TreeishPipeline: [`TreeishSugarsShared`], [`TreeishSugarsLocal`]
//!   - Stage-2 unified sugar traits: [`Stage2SugarsShared`],
//!     [`Stage2SugarsLocal`] — `Wrap`-dispatched, blanket-implemented
//!     on every `Stage2Pipeline<Base, L>`.
//!
//! Users who need only the lift-primitive surface (`Shared::wrap_init_lift`,
//! `Shared::n_lift`, `LiftBare::apply_bare`, …) can depend on `hylic`
//! alone.

#![warn(missing_docs)]

pub mod owned;
pub mod seed;
pub mod source;
pub mod stage2;
pub mod sugars;
pub mod treeish;

#[cfg(test)]
mod tests;

pub mod prelude;

pub use hylic::ops::SeedNode;
pub use owned::OwnedPipeline;
pub use seed::SeedPipeline;
pub use source::{PipelineExec, PipelineExecOnce, PipelineSourceOnce, TreeishSource};
pub use stage2::{Identity, SeedWrap, Stage2Base, Stage2Pipeline, Wrap, WrapLocal, WrapShared};
pub use sugars::{
    SeedSugarsLocal, SeedSugarsShared, Stage2SugarsLocal, Stage2SugarsShared, TreeishSugarsLocal, TreeishSugarsShared,
};
pub use treeish::TreeishPipeline;
