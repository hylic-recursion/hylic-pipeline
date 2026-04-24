//! Local-domain stage-2 sugars for `LiftedSeedPipeline`.
//!
//! Mirror of `sugars_shared.rs` with `Rc` storage and no `Send + Sync`
//! bounds. Structure and dispatch strategy are identical — only the
//! closure-storage cell type and trait bounds change. See
//! `sugars_shared.rs` for design commentary.

#![allow(missing_docs)] // module-level: public items are per-domain/per-policy mirrors of documented primitives

use std::rc::Rc;

use hylic::domain::{Domain, Local};
use hylic::ops::{ComposedLift, Lift, LiftedNode, ShapeLift};
use hylic::ops::lifted_node_internal::{self as ln_int, LiftedNodeInner};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use super::LiftedSeedPipeline;
use super::super::seed::SeedPipeline;

impl<N, Seed, H, R, L, CurN> LiftedSeedPipeline<SeedPipeline<Local, N, Seed, H, R>, L>
where N:    Clone + 'static,
      Seed: Clone + 'static,
      H:    Clone + 'static,
      R:    Clone + 'static,
      CurN: Clone + 'static,
      Local: Domain<N> + Domain<LiftedNode<N>> + Domain<LiftedNode<CurN>>,
      L:    Lift<Local, LiftedNode<N>, H, R, N2 = LiftedNode<CurN>>,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    // ── User-closure, N-aware: Node/Entry dispatch ─────────

    pub fn wrap_init<W>(self, user_wrap: W)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&CurN, &dyn Fn(&CurN) -> L::MapH) -> L::MapH + 'static,
    {
        let user = Rc::new(user_wrap);
        let lifted_w = move |ln: &LiftedNode<CurN>,
                             orig: &dyn Fn(&LiftedNode<CurN>) -> L::MapH| -> L::MapH
        {
            match ln_int::inner(ln) {
                LiftedNodeInner::Node(n) => {
                    let user = user.clone();
                    user(n, &|inner: &CurN| orig(&ln_int::node(inner.clone())))
                }
                LiftedNodeInner::Entry => orig(ln),
            }
        };
        self.then_lift(Local::wrap_init_lift::<LiftedNode<CurN>, L::MapH, L::MapR, _>(lifted_w))
    }

    pub fn memoize_by<K, KeyFn>(self, key_fn: KeyFn)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, L::MapR>>,
        >
    where K: Eq + std::hash::Hash + Clone + 'static,
          KeyFn: Fn(&CurN) -> K + 'static,
    {
        let key = Rc::new(key_fn);
        let lifted_key = move |ln: &LiftedNode<CurN>| -> Option<K> {
            match ln_int::inner(ln) {
                LiftedNodeInner::Node(n) => Some((key)(n)),
                LiftedNodeInner::Entry   => None,
            }
        };
        self.then_lift(Local::memoize_by_lift::<LiftedNode<CurN>, L::MapH, L::MapR, Option<K>, _>(lifted_key))
    }

    pub fn filter_edges<P>(self, pred: P)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, L::MapR>>,
        >
    where P: Fn(&CurN) -> bool + 'static,
    {
        let p = Rc::new(pred);
        let lifted_p = move |ln: &LiftedNode<CurN>| -> bool {
            match ln_int::inner(ln) {
                LiftedNodeInner::Node(n) => (p)(n),
                LiftedNodeInner::Entry   => true,
            }
        };
        self.then_lift(Local::filter_edges_lift::<LiftedNode<CurN>, L::MapH, L::MapR, _>(lifted_p))
    }

    // ── N-free sugars: applied uniformly ──────────────────

    pub fn wrap_accumulate<W>(self, wrapper: W)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&mut L::MapH, &L::MapR, &dyn Fn(&mut L::MapH, &L::MapR)) + 'static,
    {
        self.then_lift(Local::wrap_accumulate_lift::<LiftedNode<CurN>, L::MapH, L::MapR, _>(wrapper))
    }

    pub fn wrap_finalize<W>(self, wrapper: W)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&L::MapH, &dyn Fn(&L::MapH) -> L::MapR) -> L::MapR + 'static,
    {
        self.then_lift(Local::wrap_finalize_lift::<LiftedNode<CurN>, L::MapH, L::MapR, _>(wrapper))
    }

    pub fn zipmap<Extra, M>(self, mapper: M)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, (L::MapR, Extra)>>,
        >
    where Extra: Clone + 'static,
          M: Fn(&L::MapR) -> Extra + 'static,
    {
        self.then_lift(Local::zipmap_lift::<LiftedNode<CurN>, L::MapH, L::MapR, Extra, _>(mapper))
    }

    pub fn map_r_bi<RNew, Fwd, Bwd>(self, forward: Fwd, backward: Bwd)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>, L::MapH, RNew>>,
        >
    where RNew: Clone + 'static,
          Fwd: Fn(&L::MapR) -> RNew + 'static,
          Bwd: Fn(&RNew) -> L::MapR + 'static,
    {
        self.then_lift(Local::map_r_bi_lift::<LiftedNode<CurN>, L::MapH, L::MapR, RNew, _, _>(forward, backward))
    }

    // ── N-change sugar: map_n_bi ──────────────────────────
    //
    // Stage-2 bijective N-change on a Local-seed chain. The user's
    // (co, contra) run against the base `CurN`; the wrapper inside
    // preserves Entry as Entry and maps Node(n) ↔ Node(n2).

    pub fn map_n_bi<N2, Co, Contra>(self, co: Co, contra: Contra)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<N2>, L::MapH, L::MapR>>,
        >
    where N2: Clone + 'static,
          Co:     Fn(&CurN) -> N2 + 'static,
          Contra: Fn(&N2)   -> CurN + 'static,
          Local: Domain<LiftedNode<N2>>,
    {
        let co_rc     = Rc::new(co);
        let contra_rc = Rc::new(contra);
        let lifted_co = {
            let c = co_rc.clone();
            move |ln: &LiftedNode<CurN>| -> LiftedNode<N2> {
                match ln_int::inner(ln) {
                    LiftedNodeInner::Node(n) => ln_int::node((c)(n)),
                    LiftedNodeInner::Entry   => ln_int::entry(),
                }
            }
        };
        let lifted_contra = {
            let c = contra_rc.clone();
            move |ln: &LiftedNode<N2>| -> LiftedNode<CurN> {
                match ln_int::inner(ln) {
                    LiftedNodeInner::Node(n) => ln_int::node((c)(n)),
                    LiftedNodeInner::Entry   => ln_int::entry(),
                }
            }
        };
        self.then_lift(Local::map_n_bi_lift::<LiftedNode<CurN>, L::MapH, L::MapR, LiftedNode<N2>, _, _>(lifted_co, lifted_contra))
    }

    // ── N-parametric library lifts: explain / explain_describe ──

    pub fn explain(self)
        -> LiftedSeedPipeline<
            SeedPipeline<Local, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Local, LiftedNode<CurN>, L::MapH, L::MapR,
                                              LiftedNode<CurN>,
                                              ExplainerHeap<LiftedNode<CurN>, L::MapH,
                                                            ExplainerResult<LiftedNode<CurN>, L::MapH, L::MapR>>,
                                              ExplainerResult<LiftedNode<CurN>, L::MapH, L::MapR>>>,
        >
    {
        self.then_lift(Local::explainer_lift::<LiftedNode<CurN>, L::MapH, L::MapR>())
    }

    // `explain_describe` lives on Shared only — `Local::explainer_describe_lift`
    // does not exist. Users on the Local domain obtain streaming traces by
    // implementing the describe pattern inline or switching to Shared.
}
