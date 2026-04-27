//! `.run` / `.run_from_slice` on `Stage2Pipeline<SeedPipeline<Local, ...>, L>`.
//!
//! Mirror of `run_seed_shared.rs` with `Rc` storage and no `Send + Sync` bounds.
//! `SeedLift::new_local` / `from_rc_grow` is used in place of the Shared
//! constructors. `root_seeds` is the per-domain
//! `<Local as Domain<()>>::Graph<Seed>` (= `local_edgy::Edgy<(), Seed>`),
//! so `Seed` carries no `Send + Sync` requirement on the Local path —
//! closing the historical crack where the seed graph was Shared regardless
//! of the pipeline's domain.

use std::rc::Rc;

use hylic::domain::{Domain, Local};
use hylic::domain::local::edgy as local_edgy;
use hylic::exec::Executor;
use hylic::ops::{Lift, SeedNode, SeedLift, ShapeCapable, TreeOps};
use hylic::ops::seed_node_internal as sn_int;

use super::pipeline::Stage2Pipeline;
use crate::seed::SeedPipeline;
use crate::stage2::gat_helpers::{
    local_grow_as_rc, local_rc_as_grow,
    local_graph_as_edgy, local_edgy_as_graph,
    local_fold_as_concrete, local_concrete_as_fold,
};

impl<N, Seed, H, R, L, CurN> Stage2Pipeline<SeedPipeline<Local, N, Seed, H, R>, L>
where N:    Clone + 'static,
      Seed: Clone + 'static,
      H:    Clone + 'static,
      R:    Clone + 'static,
      CurN: Clone + 'static,
      Local: Domain<N> + Domain<SeedNode<N>> + Domain<SeedNode<CurN>> + ShapeCapable<N>,
      <Local as Domain<N>>::Grow<Seed, N>:  Clone,
      <Local as Domain<N>>::Graph<Seed>:    Clone,
      <Local as Domain<N>>::Fold<H, R>:     Clone,
      L: Lift<Local, SeedNode<N>, H, R, N2 = SeedNode<CurN>>,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    /// Run the pipeline against a `local_edgy::Edgy<(), Seed>`
    /// callback-iterator of root seeds, with the given base
    /// `entry_heap: H` for EntryRoot's initial state. Seeds are captured
    /// into the constructed `SeedLift` at this moment and consumed
    /// during execution.
    pub fn run<E>(
        &self,
        exec:       &E,
        root_seeds: <Local as Domain<()>>::Graph<Seed>,
        entry_heap: H,
    ) -> L::MapR
    where E: Executor<SeedNode<CurN>, L::MapR, Local,
                      <Local as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>>,
          <Local as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>: TreeOps<SeedNode<CurN>>,
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
                    exec.run(&fold_final, &tree_final, &sn_int::entry_root::<CurN>())
                })
            },
        )
    }

    /// Sugar: wraps a `&[Seed]` slice into the canonical
    /// `local_edgy::Edgy<(), Seed>` callback-iterator and calls `run`.
    pub fn run_from_slice<E>(
        &self,
        exec:       &E,
        seeds:      &[Seed],
        entry_heap: H,
    ) -> L::MapR
    where E: Executor<SeedNode<CurN>, L::MapR, Local,
                      <Local as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>>,
          <Local as Domain<SeedNode<CurN>>>::Graph<SeedNode<CurN>>: TreeOps<SeedNode<CurN>>,
    {
        let owned: Vec<Seed> = seeds.to_vec();
        let es: local_edgy::Edgy<(), Seed> = local_edgy::edgy_visit(
            move |_: &(), cb: &mut dyn FnMut(&Seed)| {
                for s in &owned { cb(s); }
            }
        );
        self.run(exec, es, entry_heap)
    }
}
