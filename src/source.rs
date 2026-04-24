//! Pipeline source traits and execution extensions.
//!
//! The source hierarchy has two axes: **by-reference vs by-value**
//! (consume semantics) and **seedless vs seeded** (Entry-dispatch
//! capability).
//!
//!                  seedless              seeded
//!                  ─────────             ─────────────────
//!   by-reference   TreeishSource   ◀─── SeedSource
//!                                       (extends TreeishSource)
//!   by-value       PipelineSourceOnce
//!
//! `TreeishSource` is the common supertrait for by-reference
//! pipelines: yields `(treeish, fold)` in the domain's native
//! storage. No Seed semantics.
//!
//! `SeedSource: TreeishSource` extends with a `Seed` type and a
//! `with_seeded` yield that provides `(grow, treeish, fold)` — the
//! triple SeedLift needs to build Entry/Seed/Node dispatch.
//!
//! Execution:
//!   - `PipelineExec: TreeishSource`    — `run_from_node` (any pipeline)
//!   - `PipelineExecSeed: SeedSource`   — `run` + `run_from_slice` (SeedLift-composing)
//!   - `PipelineExecOnce: PipelineSourceOnce` — by-value analogue
//!
//! Pipelines that don't carry Seed semantics (TreeishPipeline) do
//! NOT inherit `.run(...)` — that's a compile-time guarantee.

use std::sync::Arc;
use hylic::exec::Executor;
use hylic::domain::{Domain, Shared};
use hylic::graph::{self, Edgy, Treeish};
use hylic::ops::{Lift, LiftedNode, SeedLift, TreeOps};

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

// ── SeedSource ────────────────────────────────────────

/// Extends `TreeishSource` with a `Seed` type and a 3-slot yield
/// `(grow, treeish, fold)`. Implemented by pipelines that can
/// compose SeedLift for Entry dispatch.
// ANCHOR: seed_source_trait
pub trait SeedSource: TreeishSource {
    /// Reference type resolved into `Self::N` by `grow`.
    type Seed: Clone + 'static;

    /// Yield the pipeline's `(grow, treeish, fold)` triple to the
    /// given continuation. The yielded values live only for the
    /// duration of the call.
    fn with_seeded<T>(
        &self,
        cont: impl FnOnce(
            <Self::Domain as Domain<Self::N>>::Grow<Self::Seed, Self::N>,
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

// ANCHOR_END: seed_source_trait

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

// ANCHOR_END: treeish_source_trait

// ANCHOR_END: pipeline_exec_trait

impl<P: TreeishSource> PipelineExec for P {}

// ── PipelineExecSeed ──────────────────────────────────

/// Entry-dispatch execution. Only available on `SeedSource` pipelines.
// ANCHOR: pipeline_exec_seed_trait
pub trait PipelineExecSeed: SeedSource {
    /// Run from entry seeds via a finishing `SeedLift`. Shared-pinned.
    fn run<E>(
        &self,
        exec:        &E,
        entry_seeds: Edgy<(), Self::Seed>,
        entry_heap:  Self::H,
    ) -> Self::R
    where Self: SeedSource<Domain = Shared>,
          E: Executor<
            LiftedNode<Self::N>, Self::R,
            Shared, Treeish<LiftedNode<Self::N>>>,
          Self::Seed: Send + Sync,
          Self::N:    Send + Sync,
          Self::H:    Send + Sync,
    {
        self.with_seeded(|grow, treeish, fold| {
            let grow: Arc<dyn Fn(&Self::Seed) -> Self::N + Send + Sync> = grow;
            let sl: SeedLift<Self::N, Self::Seed, Self::H> =
                SeedLift::from_arc_grow(grow, entry_seeds, move || entry_heap.clone());
            sl.apply(treeish, fold, |lifted_treeish, lifted_fold| {
                exec.run(&lifted_fold, &lifted_treeish, &LiftedNode::Entry)
            })
        })
    }

    /// Sugar: wraps a `&[Seed]` slice into the canonical
    /// `Edgy<(), Seed>` callback-iterator form.
    fn run_from_slice<E>(
        &self,
        exec:       &E,
        seeds:      &[Self::Seed],
        entry_heap: Self::H,
    ) -> Self::R
    where Self: SeedSource<Domain = Shared>,
          E: Executor<
            LiftedNode<Self::N>, Self::R,
            Shared, Treeish<LiftedNode<Self::N>>>,
          Self::Seed: Send + Sync,
          Self::N:    Send + Sync,
          Self::H:    Send + Sync,
    {
        let owned: Vec<Self::Seed> = seeds.to_vec();
        let entry_seeds: Edgy<(), Self::Seed> = graph::edgy_visit(
            move |_: &(), cb: &mut dyn FnMut(&Self::Seed)| {
                for s in &owned { cb(s); }
            }
        );
        self.run(exec, entry_seeds, entry_heap)
    }
}

// ANCHOR_END: pipeline_exec_seed_trait

impl<P: SeedSource> PipelineExecSeed for P {}

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
