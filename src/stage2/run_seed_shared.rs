//! `.run` / `.run_from_slice` on `Stage2Pipeline<SeedPipeline<Shared, ...>, L>`.
//!
//! At `.run` time, `SeedLift` is assembled from the `SeedPipeline`'s
//! `grow` + `seeds_from_node` and the user's `root_seeds` +
//! `entry_heap`. The composition is:
//!
//! ```text
//! base.fuse(grow, seeds_from_node)                   — treeish<N>
//!               │
//!               │ SeedLift::apply  (N → SeedNode<N>)
//!               ▼
//!       (treeish<SeedNode<N>>,  fold<SeedNode<N>, H, R>)
//!               │
//!               │ pre_lift.apply  (the stored user chain)
//!               ▼
//!       (treeish<L::N2>,          fold<L::N2, L::MapH, L::MapR>)
//!               │
//!               │ exec.run(&f, &t, &SeedNode::entry_root())
//!               ▼
//!               L::MapR
//! ```

use std::sync::Arc;

use hylic::domain::{Domain, Shared};
use hylic::exec::Executor;
use hylic::graph::{self, Edgy};
use hylic::ops::{Lift, SeedNode, SeedLift, ShapeCapable, TreeOps};
use hylic::ops::seed_node_internal as sn_int;

use super::pipeline::Stage2Pipeline;
use crate::seed::SeedPipeline;
use crate::stage2::gat_helpers::{
    shared_grow_as_arc, shared_arc_as_grow,
    shared_graph_as_edgy, shared_edgy_as_graph,
    shared_fold_as_concrete, shared_concrete_as_fold,
};

impl<N, Seed, H, R, L, CurN> Stage2Pipeline<SeedPipeline<Shared, N, Seed, H, R>, L>
where N:    Clone + Send + Sync + 'static,
      Seed: Clone + Send + Sync + 'static,
      H:    Clone + Send + Sync + 'static,
      R:    Clone + Send + Sync + 'static,
      CurN: Clone + Send + Sync + 'static,
      Shared: Domain<N> + Domain<SeedNode<N>> + Domain<SeedNode<CurN>> + ShapeCapable<N>,
      <Shared as Domain<N>>::Grow<Seed, N>:  Clone,
      <Shared as Domain<N>>::Graph<Seed>:    Clone,
      <Shared as Domain<N>>::Fold<H, R>:     Clone,
      L: Lift<Shared, SeedNode<N>, H, R, N2 = SeedNode<CurN>>,
      L::MapH: Clone + Send + Sync + 'static,
      L::MapR: Clone + Send + Sync + 'static,
{
    /// Run the pipeline against an `Edgy<(), Seed>` callback-iterator
    /// of root seeds, with the given base `entry_heap: H` for EntryRoot's
    /// initial state. Seeds are captured into the constructed
    /// `SeedLift` at this moment and consumed during execution.
    pub fn run<E>(
        &self,
        exec:       &E,
        root_seeds: Edgy<(), Seed>,
        entry_heap: H,
    ) -> L::MapR
    where E: Executor<SeedNode<CurN>, L::MapR, Shared,
                      <Shared as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>>,
          <Shared as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>: TreeOps<SeedNode<CurN>>,
    {
        let grow_abs = self.base.grow.clone();
        let grow_arc: Arc<dyn Fn(&Seed) -> N + Send + Sync> =
            shared_grow_as_arc::<Seed, N>(grow_abs);
        let sl: SeedLift<Shared, N, Seed, H> = SeedLift::from_arc_grow(
            grow_arc.clone(),
            root_seeds,
            move || entry_heap.clone(),
        );
        let base_treeish_abstract = Shared::fuse_grow_with_seeds::<Seed>(
            shared_arc_as_grow::<Seed, N>(grow_arc),
            self.base.seeds_from_node.clone(),
        );
        let base_treeish_concrete = shared_graph_as_edgy::<N>(base_treeish_abstract);
        let base_fold_concrete = shared_fold_as_concrete::<N, H, R>(self.base.fold.clone());
        sl.apply(
            shared_edgy_as_graph::<N>(base_treeish_concrete),
            shared_concrete_as_fold::<N, H, R>(base_fold_concrete),
            |lt, lf| {
                self.pre_lift.apply(lt, lf, |tree_final, fold_final| {
                    exec.run(&fold_final, &tree_final, &sn_int::entry_root::<CurN>())
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
    where E: Executor<SeedNode<CurN>, L::MapR, Shared,
                      <Shared as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>>,
          <Shared as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>: TreeOps<SeedNode<CurN>>,
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
