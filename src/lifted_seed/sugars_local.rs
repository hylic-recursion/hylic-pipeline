//! Local-domain stage-2 sugars for `LiftedSeedPipeline`.
//!
//! Mirror of `sugars_shared.rs` with `Rc` storage and no `Send + Sync`
//! bounds. Structure and dispatch strategy are identical — only the
//! closure-storage cell type and trait bounds change. See
//! `sugars_shared.rs` for design commentary.

#![allow(missing_docs)] // module-level: public items are per-domain/per-policy mirrors of documented primitives

use std::rc::Rc;

use hylic::domain::{Domain, Local};
use hylic::ops::{ComposedLift, Lift, SeedNode, ShapeLift};
use hylic::ops::seed_node_internal::{self as sn_int, SeedNodeInner};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use super::LiftedSeedPipeline;
use super::super::seed::SeedPipeline;

impl<N, Seed, H, R, L, CurN> LiftedSeedPipeline<SeedPipeline<Local, N, Seed, H, R>, L>
where N:    Clone + 'static,
      Seed: Clone + 'static,
      H:    Clone + 'static,
      R:    Clone + 'static,
      CurN: Clone + 'static,
      Local: Domain<N> + Domain<SeedNode<N>> + Domain<SeedNode<CurN>>,
      L:    Lift<Local, SeedNode<N>, H, R, N2 = SeedNode<CurN>>,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    // ── User-closure, N-aware: Node/Entry dispatch ─────────

    pub fn wrap_init<W>(self, user_wrap: W)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&CurN, &dyn Fn(&CurN) -> L::MapH) -> L::MapH + 'static,
    {
        let user = Rc::new(user_wrap);
        let lifted_w = move |ln: &SeedNode<CurN>,
                             orig: &dyn Fn(&SeedNode<CurN>) -> L::MapH| -> L::MapH
        {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => {
                    let user = user.clone();
                    user(n, &|inner: &CurN| orig(&sn_int::node(inner.clone())))
                }
                SeedNodeInner::EntryRoot => orig(ln),
            }
        };
        self.then_lift(Local::wrap_init_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(lifted_w))
    }

    pub fn memoize_by<K, KeyFn>(self, key_fn: KeyFn)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where K: Eq + std::hash::Hash + Clone + 'static,
          KeyFn: Fn(&CurN) -> K + 'static,
    {
        let key = Rc::new(key_fn);
        let lifted_key = move |ln: &SeedNode<CurN>| -> Option<K> {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => Some((key)(n)),
                SeedNodeInner::EntryRoot   => None,
            }
        };
        self.then_lift(Local::memoize_by_lift::<SeedNode<CurN>, L::MapH, L::MapR, Option<K>, _>(lifted_key))
    }

    pub fn filter_edges<P>(self, pred: P)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where P: Fn(&CurN) -> bool + 'static,
    {
        let p = Rc::new(pred);
        let lifted_p = move |ln: &SeedNode<CurN>| -> bool {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => (p)(n),
                SeedNodeInner::EntryRoot   => true,
            }
        };
        self.then_lift(Local::filter_edges_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(lifted_p))
    }

    // ── N-free sugars: applied uniformly ──────────────────

    pub fn wrap_accumulate<W>(self, wrapper: W)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&mut L::MapH, &L::MapR, &dyn Fn(&mut L::MapH, &L::MapR)) + 'static,
    {
        self.then_lift(Local::wrap_accumulate_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(wrapper))
    }

    pub fn wrap_finalize<W>(self, wrapper: W)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&L::MapH, &dyn Fn(&L::MapH) -> L::MapR) -> L::MapR + 'static,
    {
        self.then_lift(Local::wrap_finalize_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(wrapper))
    }

    pub fn zipmap<Extra, M>(self, mapper: M)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, (L::MapR, Extra)>>,
        >
    where Extra: Clone + 'static,
          M: Fn(&L::MapR) -> Extra + 'static,
    {
        self.then_lift(Local::zipmap_lift::<SeedNode<CurN>, L::MapH, L::MapR, Extra, _>(mapper))
    }

    pub fn map_r_bi<RNew, Fwd, Bwd>(self, forward: Fwd, backward: Bwd)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>, L::MapH, RNew>>,
        >
    where RNew: Clone + 'static,
          Fwd: Fn(&L::MapR) -> RNew + 'static,
          Bwd: Fn(&RNew) -> L::MapR + 'static,
    {
        self.then_lift(Local::map_r_bi_lift::<SeedNode<CurN>, L::MapH, L::MapR, RNew, _, _>(forward, backward))
    }

    // ── N-change sugar: map_n_bi ──────────────────────────
    //
    // Stage-2 bijective N-change on a Local-seed chain. The user's
    // (co, contra) run against the base `CurN`; the wrapper inside
    // preserves Entry as Entry and maps Node(n) ↔ Node(n2).

    pub fn map_n_bi<N2, Co, Contra>(self, co: Co, contra: Contra)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<N2>, L::MapH, L::MapR>>,
        >
    where N2: Clone + 'static,
          Co:     Fn(&CurN) -> N2 + 'static,
          Contra: Fn(&N2)   -> CurN + 'static,
          Local: Domain<SeedNode<N2>>,
    {
        let co_rc     = Rc::new(co);
        let contra_rc = Rc::new(contra);
        let lifted_co = {
            let c = co_rc.clone();
            move |ln: &SeedNode<CurN>| -> SeedNode<N2> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot   => sn_int::entry_root(),
                }
            }
        };
        let lifted_contra = {
            let c = contra_rc.clone();
            move |ln: &SeedNode<N2>| -> SeedNode<CurN> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot   => sn_int::entry_root(),
                }
            }
        };
        self.then_lift(Local::map_n_bi_lift::<SeedNode<CurN>, L::MapH, L::MapR, SeedNode<N2>, _, _>(lifted_co, lifted_contra))
    }

    // ── N-parametric library lifts: explain / explain_describe ──

    pub fn explain(self)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, SeedNode<CurN>, L::MapH, L::MapR,
                                              SeedNode<CurN>,
                                              ExplainerHeap<SeedNode<CurN>, L::MapH,
                                                            ExplainerResult<SeedNode<CurN>, L::MapH, L::MapR>>,
                                              ExplainerResult<SeedNode<CurN>, L::MapH, L::MapR>>>,
        >
    {
        self.then_lift(Local::explainer_lift::<SeedNode<CurN>, L::MapH, L::MapR>())
    }

    // `explain_describe` lives on Shared only — `Local::explainer_describe_lift`
    // does not exist. Users on the Local domain obtain streaming traces by
    // implementing the describe pattern inline or switching to Shared.
}
