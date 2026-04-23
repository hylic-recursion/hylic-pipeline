//! hylic-pipeline prelude. Extends `hylic::prelude::*` with pipeline
//! typestates and sugar traits.
//!
//! ```no_run
//! use hylic_pipeline::prelude::*;
//! ```
//!
//! gives: everything in hylic's core prelude (Shared Fold/Edgy
//! constructors, executor helpers, lift atoms, explainer helpers)
//! PLUS pipeline types (`SeedPipeline`, `LiftedPipeline`, …), source
//! traits (`TreeishSource`, `PipelineExecSeed`), and the Shared sugar
//! trait (`LiftedSugarsShared`).
//!
//! For Local pipelines, also `use hylic_pipeline::prelude::local::*;`.

pub use hylic::prelude::*;

pub use crate::{
    SeedPipeline, TreeishPipeline, LiftedPipeline, OwnedPipeline,
    TreeishSource, SeedSource,
    PipelineExec, PipelineExecSeed, PipelineExecOnce,
    PipelineSourceOnce,
    LiftedSugarsShared,
};

pub mod local {
    //! Local pipeline extras (add to `prelude::*`).
    pub use hylic::prelude::local::*;
    pub use crate::LiftedSugarsLocal;
}

pub mod owned {
    //! Owned pipeline extras (add to `prelude::*`).
    pub use hylic::prelude::owned::*;
}
