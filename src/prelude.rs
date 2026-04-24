//! hylic-pipeline prelude. Extends `hylic::prelude::*` with pipeline
//! typestates and sugar traits.
//!
//! ```no_run
//! use hylic_pipeline::prelude::*;
//! ```
//!
//! gives: everything in hylic's core prelude (domain markers,
//! Shared-default Fold/Edgy constructors, executor helpers, lift
//! atoms, explainer helpers) PLUS pipeline types
//! (`SeedPipeline`, `LiftedPipeline`, `LiftedSeedPipeline`,
//! `TreeishPipeline`, `OwnedPipeline`), source traits
//! (`TreeishSource`, `PipelineExec`), and the sugar traits.

pub use hylic::prelude::*;

pub use crate::{
    SeedPipeline, TreeishPipeline, LiftedPipeline, LiftedSeedPipeline, OwnedPipeline,
    TreeishSource,
    PipelineExec, PipelineExecOnce,
    PipelineSourceOnce,
    SeedSugarsShared, SeedSugarsLocal,
    TreeishSugarsShared, TreeishSugarsLocal,
    LiftedSugarsShared, LiftedSugarsLocal,
};
