//! Pipeline source traits and execution extensions.
//!
//! Under Option B (post-`project-entry-refactor`), the seed path is
//! a distinct type (`Stage2Pipeline`) with inherent methods, not
//! a trait-level concern. `source.rs` carries only the seedless
//! abstractions:
//!
//!   - `TreeishSource`              — yield `(treeish, fold)` by reference
//!   - `PipelineSourceOnce`         — yield them by value (consuming)
//!   - `PipelineExec: TreeishSource` — `run_from_node`
//!   - `PipelineExecOnce: PipelineSourceOnce` — `run_from_node_once`
//!
//! `SeedSource` / `PipelineExecSeed` / `with_seeded` have been
//! removed: `SeedPipeline::lift()` transitions directly to
//! `Stage2Pipeline`, whose `.run` captures
//! seeds and assembles `SeedLift` as the first lift in the chain.

use hylic::exec::Executor;
use hylic::domain::Domain;
use hylic::ops::TreeOps;

// ── TreeishSource ─────────────────────────────────────

/// A by-reference pipeline that yields `(treeish, fold)` for
/// execution. Seed-agnostic.
// ANCHOR: treeish_source_trait
pub trait TreeishSource {
    /// Domain in which the pipeline's slots are stored.
    type Domain: Domain<Self::N>;
    /// Node type flowing through the fold and graph.
    type N: Clone + 'static;
    /// Per-node heap type used by the fold.
    type H: Clone + 'static;
    /// Result type returned at each fold node.
    type R: Clone + 'static;

    /// Yield the pipeline's `(treeish, fold)` pair to the given
    /// continuation. The yielded values may be borrowed only for the
    /// duration of the continuation.
    fn with_treeish<T>(
        &self,
        cont: impl FnOnce(
            <Self::Domain as Domain<Self::N>>::Graph<Self::N>,
            <Self::Domain as Domain<Self::N>>::Fold<Self::H, Self::R>,
        ) -> T,
    ) -> T;
}

// ── PipelineSourceOnce ────────────────────────────────

/// By-value analogue of [`TreeishSource`], implemented by one-shot
/// pipelines such as `OwnedPipeline`. Seedless.
pub trait PipelineSourceOnce {
    /// Domain in which the pipeline's slots are stored.
    type Domain: Domain<Self::N>;
    /// Node type flowing through the fold and graph.
    type N:    'static;
    /// Per-node heap type used by the fold.
    type H:    'static;
    /// Result type returned at each fold node.
    type R:    'static;

    /// Consume the pipeline and yield its `(treeish, fold)` pair to
    /// the given continuation.
    fn with_constructed_once<T>(
        self,
        cont: impl FnOnce(
            <Self::Domain as Domain<Self::N>>::Graph<Self::N>,
            <Self::Domain as Domain<Self::N>>::Fold<Self::H, Self::R>,
        ) -> T,
    ) -> T;
}

// ANCHOR_END: treeish_source_trait

// ── PipelineExec ──────────────────────────────────────

/// Run-from-root execution on any `TreeishSource`.
// ANCHOR: pipeline_exec_trait
pub trait PipelineExec: TreeishSource {
    /// Execute the pipeline from the given `root` node under the
    /// supplied executor and return the root's fold result.
    fn run_from_node<E>(
        &self,
        exec: &E,
        root: &Self::N,
    ) -> Self::R
    where E: Executor<
            Self::N, Self::R, Self::Domain,
            <Self::Domain as Domain<Self::N>>::Graph<Self::N>,
        >,
          <Self::Domain as Domain<Self::N>>::Graph<Self::N>: TreeOps<Self::N>,
    {
        self.with_treeish(|treeish, fold| {
            exec.run(&fold, &treeish, root)
        })
    }
}

// ANCHOR_END: pipeline_exec_trait

impl<P: TreeishSource> PipelineExec for P {}

// ── PipelineExecOnce ──────────────────────────────────

/// By-value execution for `PipelineSourceOnce` sources (notably
/// `OwnedPipeline`). Consumes `self`.
pub trait PipelineExecOnce: PipelineSourceOnce + Sized {
    /// Consume the pipeline, apply it at `root` under the given
    /// executor, and return the root's fold result.
    fn run_from_node_once<E>(
        self,
        exec: &E,
        root: &Self::N,
    ) -> Self::R
    where E: Executor<
            Self::N, Self::R, Self::Domain,
            <Self::Domain as Domain<Self::N>>::Graph<Self::N>,
        >,
          <Self::Domain as Domain<Self::N>>::Graph<Self::N>: TreeOps<Self::N>,
          Self::N: Clone,
    {
        self.with_constructed_once(|treeish, fold| {
            exec.run(&fold, &treeish, root)
        })
    }
}

impl<P: PipelineSourceOnce + Sized> PipelineExecOnce for P {}
