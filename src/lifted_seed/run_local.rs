//! `.run` / `.run_from_slice` on `LiftedSeedPipeline` — Local domain.
//!
//! Mirror of `run.rs` with `Rc` storage and no `Send + Sync` bounds.
//! `SeedLift::new_local` / `from_rc_grow` is used in place of the
//! Shared constructors. See `run.rs` for the composition flow.

use std::rc::Rc;

use hylic::domain::{Domain, Local};
use hylic::exec::Executor;
use hylic::graph::{self, Edgy};
use hylic::ops::{Lift, LiftedNode, SeedLift, ShapeCapable, TreeOps};
use hylic::ops::lifted_node_internal as ln_int;

use super::LiftedSeedPipeline;
use super::super::seed::SeedPipeline;
use super::gat_helpers::{
    local_grow_as_rc, local_rc_as_grow,
    local_graph_as_edgy, local_edgy_as_graph,
    local_fold_as_concrete, local_concrete_as_fold,
};

// `Seed: Send + Sync` is required because `SeedLift`'s `entry_seeds`
// field is a Shared-domain `Edgy<(), Seed>` regardless of the
// pipeline's domain (the callback-iterator shape is the library's
// single seed-iteration protocol). Local-domain `N`, `H`, `R`, and
// `CurN` retain no `Send + Sync` requirement.
impl<N, Seed, H, R, L, CurN> LiftedSeedPipeline<SeedPipeline<Local, N, Seed, H, R>, L>
where N:    Clone + 'static,
      Seed: Clone + Send + Sync + 'static,
      H:    Clone + 'static,
      R:    Clone + 'static,
      CurN: Clone + 'static,
      Local: Domain<N> + Domain<LiftedNode<N>> + Domain<LiftedNode<CurN>> + ShapeCapable<N>,
      <Local as Domain<N>>::Grow<Seed, N>:  Clone,
      <Local as Domain<N>>::Graph<Seed>:    Clone,
      <Local as Domain<N>>::Fold<H, R>:     Clone,
      L: Lift<Local, LiftedNode<N>, H, R, N2 = LiftedNode<CurN>>,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
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
    where E: Executor<LiftedNode<CurN>, L::MapR, Local,
                      <Local as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>>,
          <Local as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>: TreeOps<LiftedNode<CurN>>,
    {
        let grow_abs = self.base.grow.clone();
        let grow_rc: Rc<dyn Fn(&Seed) -> N> = local_grow_as_rc::<Seed, N>(grow_abs);
        let sl: SeedLift<Local, N, Seed, H> = SeedLift::from_rc_grow(
            grow_rc.clone(),
            root_seeds,
            move || entry_heap.clone(),
        );
        let base_treeish_abstract = Local::fuse_grow_with_seeds::<Seed>(
            local_rc_as_grow::<Seed, N>(grow_rc),
            self.base.seeds_from_node.clone(),
        );
        let base_treeish_concrete = local_graph_as_edgy::<N>(base_treeish_abstract);
        let base_fold_concrete = local_fold_as_concrete::<N, H, R>(self.base.fold.clone());
        sl.apply(
            local_edgy_as_graph::<N>(base_treeish_concrete),
            local_concrete_as_fold::<N, H, R>(base_fold_concrete),
            |lt, lf| {
                self.pre_lift.apply(lt, lf, |tree_final, fold_final| {
                    exec.run(&fold_final, &tree_final, &ln_int::entry::<CurN>())
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
    where E: Executor<LiftedNode<CurN>, L::MapR, Local,
                      <Local as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>>,
          <Local as Domain<LiftedNode<CurN>>>::Graph<LiftedNode<CurN>>: TreeOps<LiftedNode<CurN>>,
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
