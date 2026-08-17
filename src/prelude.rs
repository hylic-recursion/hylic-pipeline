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
//! and sugar traits (`SeedSugars*`, `TreeishSugars*`, `Stage2Sugars*`).

pub use hylic::prelude::*;

pub use crate::{
    OwnedPipeline, PipelineExec, PipelineExecOnce, PipelineSourceOnce, SeedPipeline, SeedSugarsLocal, SeedSugarsShared,
    Stage2Pipeline, Stage2SugarsLocal, Stage2SugarsShared, TreeishPipeline, TreeishSource, TreeishSugarsLocal,
    TreeishSugarsShared,
};
