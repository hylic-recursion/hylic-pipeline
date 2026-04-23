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
//! (`SeedPipeline`, `LiftedPipeline`, …), source traits
//! (`TreeishSource`, `PipelineExecSeed`), and both sugar traits
//! (`LiftedSugarsShared`, `LiftedSugarsLocal`).

pub use hylic::prelude::*;

pub use crate::{
    SeedPipeline, TreeishPipeline, LiftedPipeline, OwnedPipeline,
    TreeishSource, SeedSource,
    PipelineExec, PipelineExecSeed, PipelineExecOnce,
    PipelineSourceOnce,
    LiftedSugarsShared, LiftedSugarsLocal,
};
