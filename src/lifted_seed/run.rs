//! `.run` / `.run_from_slice` on `LiftedSeedPipeline`. Shared domain.
//!
//! At `.run` time, `SeedLift` is assembled from the `SeedPipeline`'s
//! `grow` + `seeds_from_node` and the user's `root_seeds` +
//! `entry_heap`. The composition is:
//!
//!     base.fuse(grow, seeds_from_node)                   — treeish<N>
//!                   │
//!                   │ SeedLift::apply  (N → LiftedNode<N>)
//!                   ▼
//!           (treeish<LiftedNode<N>>,  fold<LiftedNode<N>, H, R>)
//!                   │
//!                   │ pre_lift.apply  (the stored user chain)
//!                   ▼
//!           (treeish<L::N2>,          fold<L::N2, L::MapH, L::MapR>)
//!                   │
//!                   │ exec.run(&f, &t, &LiftedNode::Entry)
//!                   ▼
//!                   L::MapR

use std::sync::Arc;

use hylic::domain::{Domain, Shared};
use hylic::domain::shared::fold::Fold;
use hylic::exec::Executor;
use hylic::graph::{self, Edgy, Treeish};
use hylic::ops::{Lift, LiftedNode, SeedLift, ShapeCapable, TreeOps};
use super::LiftedSeedPipeline;
use super::super::seed::SeedPipeline;

// ── Shared-pinned normalisation helpers ──────────────────
// The GAT `<Shared as Domain<N>>::Grow<Seed, N>` doesn't reduce to its
// concrete carrier (`Arc<dyn Fn + Send + Sync>`) inside a generic impl
// body. Pinning Self = Shared via a free fn lets the GAT normalise in
// the fn's scope. Same trick used in hylic/src/domain/shared/shape_capable.rs.

#[inline]
fn shared_grow_as_arc<Seed: 'static, NOut: 'static>(
    g: <Shared as Domain<NOut>>::Grow<Seed, NOut>,
) -> Arc<dyn Fn(&Seed) -> NOut + Send + Sync> {
    g
}

#[inline]
fn shared_graph_as_edgy<NodeT: 'static>(
    g: <Shared as Domain<NodeT>>::Graph<NodeT>,
) -> Edgy<NodeT, NodeT> {
    g
}

#[inline]
fn shared_fold_as_concrete<NodeT: 'static, H: 'static, R: 'static>(
    f: <Shared as Domain<NodeT>>::Fold<H, R>,
) -> Fold<NodeT, H, R> {
    f
}

// Reverse direction: concrete → abstract GAT, Shared-pinned.

#[inline]
fn shared_arc_as_grow<Seed: 'static, NOut: 'static>(
    a: Arc<dyn Fn(&Seed) -> NOut + Send + Sync>,
) -> <Shared as Domain<NOut>>::Grow<Seed, NOut> {
    a
}

#[inline]
fn shared_edgy_as_graph<NodeT: 'static>(
    e: Edgy<NodeT, NodeT>,
) -> <Shared as Domain<NodeT>>::Graph<NodeT> {
    e
}

#[inline]
fn shared_concrete_as_fold<NodeT: 'static, H: 'static, R: 'static>(
    f: Fold<NodeT, H, R>,
) -> <Shared as Domain<NodeT>>::Fold<H, R> {
    f
}

impl<N, Seed, H, R, L, CurN> LiftedSeedPipeline<SeedPipeline<Shared, N, Seed, H, R>, L>
where N:    Clone + Send + Sync + 'static,
      Seed: Clone + Send + Sync + 'static,
      H:    Clone + Send + Sync + 'static,
      R:    Clone + Send + Sync + 'static,
      CurN: Clone + Send + Sync + 'static,
      Shared: Domain<N> + Domain<LiftedNode<N>> + Domain<LiftedNode<CurN>> + ShapeCapable<N>,
      <Shared as Domain<N>>::Grow<Seed, N>:  Clone,
      <Shared as Domain<N>>::Graph<Seed>:    Clone,
      <Shared as Domain<N>>::Fold<H, R>:     Clone,
      L: Lift<Shared, LiftedNode<N>, H, R, N2 = LiftedNode<CurN>>,
      L::MapH: Clone + Send + Sync + 'static,
      L::MapR: Clone + Send + Sync + 'static,
{
    /// Run the pipeline against an `Edgy<(), Seed>` callback-iterator
    /// of root seeds, with the given base `entry_heap: H` for Entry's
    /// initial state. Seeds are captured into the constructed
    /// `SeedLift` at this moment and consumed during execution.
    pub fn run<E>(
        &self,
        exec:       &E,
        root_seeds: Edgy<(), Seed>,
        entry_heap: H,
    ) -> L::MapR
    where E: Executor<LiftedNode<CurN>, L::MapR, Shared,
                      <Shared as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>>,
          <Shared as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>: TreeOps<LiftedNode<CurN>>,
    {
        let grow_abs = self.base.grow.clone();
        let grow_arc: Arc<dyn Fn(&Seed) -> N + Send + Sync> =
            shared_grow_as_arc::<Seed, N>(grow_abs);
        let sl: SeedLift<N, Seed, H> = SeedLift::from_arc_grow(
            grow_arc.clone(),
            root_seeds,
            move || entry_heap.clone(),
        );
        // Re-pack the Arc back into the abstract GAT shape for
        // fuse_grow_with_seeds. Both layers are the same Arc; the
        // wrapping only satisfies Rust's type checker.
        let base_treeish_abstract = Shared::fuse_grow_with_seeds::<Seed>(
            shared_arc_as_grow::<Seed, N>(grow_arc),
            self.base.seeds_from_node.clone(),
        );
        let base_treeish_concrete: Treeish<N> =
            shared_graph_as_edgy::<N>(base_treeish_abstract);
        let base_fold_concrete: Fold<N, H, R> =
            shared_fold_as_concrete::<N, H, R>(self.base.fold.clone());
        // Pack back into the GAT shape SeedLift::apply expects.
        sl.apply(
            shared_edgy_as_graph::<N>(base_treeish_concrete),
            shared_concrete_as_fold::<N, H, R>(base_fold_concrete),
            |lt, lf| {
                self.pre_lift.apply(lt, lf, |tree_final, fold_final| {
                    exec.run(&fold_final, &tree_final, &LiftedNode::Entry)
                })
            },
        )
    }

    /// Sugar: wraps a `&[Seed]` slice into the canonical
    /// `Edgy<(), Seed>` callback-iterator and calls `run`.
    pub fn run_from_slice<E>(
        &self,
        exec:       &E,
        seeds:      &[Seed],
        entry_heap: H,
    ) -> L::MapR
    where E: Executor<LiftedNode<CurN>, L::MapR, Shared,
                      <Shared as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>>,
          <Shared as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>: TreeOps<LiftedNode<CurN>>,
    {
        let owned: Vec<Seed> = seeds.to_vec();
        let es: Edgy<(), Seed> = graph::edgy_visit(
            move |_: &(), cb: &mut dyn FnMut(&Seed)| {
                for s in &owned { cb(s); }
            }
        );
        self.run(exec, es, entry_heap)
    }
}
