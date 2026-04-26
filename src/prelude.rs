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
//! (`SeedPipeline`, `Stage2Pipeline`, `TreeishPipeline`,
//! `OwnedPipeline`), source traits (`TreeishSource`, `PipelineExec`),
//! and sugar traits (`SeedSugars*`, `TreeishSugars*`, `LiftedSugars*`).

pub use hylic::prelude::*;

#[allow(deprecated)]
pub use crate::{
    SeedPipeline, TreeishPipeline, Stage2Pipeline,
    LiftedPipeline, LiftedSeedPipeline,  // deprecated aliases of Stage2Pipeline
    OwnedPipeline,
    TreeishSource,
    PipelineExec, PipelineExecOnce,
    PipelineSourceOnce,
    SeedSugarsShared, SeedSugarsLocal,
    TreeishSugarsShared, TreeishSugarsLocal,
    LiftedSugarsShared, LiftedSugarsLocal,
};
