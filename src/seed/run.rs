//! `SeedPipeline::run` / `SeedPipeline::run_from_slice` shorthand.
//!
//! Forwards to `self.clone().lift().run(...)`. The explicit
//! `.lift()` + `Stage2Pipeline` typestate is unchanged; this just
//! elides the empty `.lift()` step at every call site that adds no
//! Stage-2 sugars.
//!
//! Per-domain (Shared / Local), because the bound list mirrors the
//! Stage-2 unified run at `L = IdentityLift` and the executor
//! signature is concrete on the user-visible side.

use hylic::domain::{Domain, Local, Shared};
use hylic::exec::Executor;
use hylic::ops::{SeedNode, ShapeCapable, TreeOps};

use super::SeedPipeline;

impl<N, Seed, H, R> SeedPipeline<Shared, N, Seed, H, R>
where
    N: Clone + Send + Sync + 'static,
    Seed: Clone + Send + Sync + 'static,
    H: Clone + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
    Shared: ShapeCapable<N>,
    <Shared as Domain<N>>::Grow<Seed, N>: Clone,
    <Shared as Domain<N>>::Graph<Seed>: Clone,
    <Shared as Domain<N>>::Fold<H, R>: Clone,
{
    /// Run the pipeline against an explicit `Edgy<(), Seed>` of root
    /// seeds and an `entry_heap: H`. Equivalent to
    /// `self.clone().lift().run(exec, (root_seeds, entry_heap))`.
    pub fn run<E>(&self, exec: &E, root_seeds: <Shared as Domain<()>>::Graph<Seed>, entry_heap: H) -> R
    where
        E: Executor<SeedNode<N>, R, Shared, <Shared as Domain<SeedNode<N>>>::Graph<SeedNode<N>>>,
        <Shared as Domain<SeedNode<N>>>::Graph<SeedNode<N>>: TreeOps<SeedNode<N>>,
    {
        self.clone()
            .lift()
            .run::<E, N>(exec, root_seeds, entry_heap)
    }

    /// Slice-of-seeds shorthand. Equivalent to
    /// `self.clone().lift().run_from_slice(exec, seeds, entry_heap)`.
    pub fn run_from_slice<E>(&self, exec: &E, seeds: &[Seed], entry_heap: H) -> R
    where
        E: Executor<SeedNode<N>, R, Shared, <Shared as Domain<SeedNode<N>>>::Graph<SeedNode<N>>>,
        <Shared as Domain<SeedNode<N>>>::Graph<SeedNode<N>>: TreeOps<SeedNode<N>>,
    {
        self.clone()
            .lift()
            .run_from_slice::<E, N>(exec, seeds, entry_heap)
    }
}

impl<N, Seed, H, R> SeedPipeline<Local, N, Seed, H, R>
where
    N: Clone + 'static,
    Seed: Clone + 'static,
    H: Clone + 'static,
    R: Clone + 'static,
    Local: ShapeCapable<N>,
    <Local as Domain<N>>::Grow<Seed, N>: Clone,
    <Local as Domain<N>>::Graph<Seed>: Clone,
    <Local as Domain<N>>::Fold<H, R>: Clone,
{
    /// Run the pipeline against an explicit `Edgy<(), Seed>` of root
    /// seeds and an `entry_heap: H`.
    pub fn run<E>(&self, exec: &E, root_seeds: <Local as Domain<()>>::Graph<Seed>, entry_heap: H) -> R
    where
        E: Executor<SeedNode<N>, R, Local, <Local as Domain<SeedNode<N>>>::Graph<SeedNode<N>>>,
        <Local as Domain<SeedNode<N>>>::Graph<SeedNode<N>>: TreeOps<SeedNode<N>>,
    {
        self.clone()
            .lift()
            .run::<E, N>(exec, root_seeds, entry_heap)
    }

    /// Slice-of-seeds shorthand.
    pub fn run_from_slice<E>(&self, exec: &E, seeds: &[Seed], entry_heap: H) -> R
    where
        E: Executor<SeedNode<N>, R, Local, <Local as Domain<SeedNode<N>>>::Graph<SeedNode<N>>>,
        <Local as Domain<SeedNode<N>>>::Graph<SeedNode<N>>: TreeOps<SeedNode<N>>,
    {
        self.clone()
            .lift()
            .run_from_slice::<E, N>(exec, seeds, entry_heap)
    }
}
