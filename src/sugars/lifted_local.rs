//! Blanket Stage-2 sugars for the Local domain. Mirror of
//! `lifted_shared.rs` with Rc storage and no Send+Sync bounds.
//!
//! With this trait in scope (`use hylic::prelude::*` or
//! `use hylic::LiftedSugarsLocal`), users can call `.wrap_init(w)`,
//! `.zipmap(m)`, `.map_r_bi(fwd, bwd)` directly on a
//! `SeedPipeline<Local, …>`, `TreeishPipeline<Local, …>`, or
//! `LiftedPipeline<…Local base…>` — no `.lift()` ceremony required
//! for Stage-1 types, no `_local` suffix clutter.

#![allow(missing_docs)] // module-level: public items are per-domain/per-policy mirrors of documented primitives

use crate::lifted::LiftedPipeline;
use crate::treeish::TreeishPipeline;
use crate::source::TreeishSource;
use hylic::domain::{Domain, Local};
use hylic::ops::{ComposedLift, IdentityLift, Lift, ShapeLift};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

pub trait LiftedSugarsLocal<N, H, R>:
    TreeishSource<Domain = Local, N = N, H = H, R = R> + Sized
where
    N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
{
    type With<L2>: TreeishSource<Domain = Local>
    where L2: Lift<Local, N, H, R>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
          Local:    Domain<L2::N2>;

    /// Sole primitive: append a lift to the chain.
    fn then_lift<L2>(self, l: L2) -> Self::With<L2>
    where L2: Lift<Local, N, H, R>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
          Local:    Domain<L2::N2>;

    // ── fold-side sugars ─────────────────────────────────────

    fn wrap_init<W>(self, wrapper: W) -> Self::With<ShapeLift<Local, N, H, R, N, H, R>>
    where W: Fn(&N, &dyn Fn(&N) -> H) -> H + 'static,
    { self.then_lift(Local::wrap_init_lift::<N, H, R, _>(wrapper)) }

    fn wrap_accumulate<W>(self, wrapper: W) -> Self::With<ShapeLift<Local, N, H, R, N, H, R>>
    where W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + 'static,
    { self.then_lift(Local::wrap_accumulate_lift::<N, H, R, _>(wrapper)) }

    fn wrap_finalize<W>(self, wrapper: W) -> Self::With<ShapeLift<Local, N, H, R, N, H, R>>
    where W: Fn(&H, &dyn Fn(&H) -> R) -> R + 'static,
    { self.then_lift(Local::wrap_finalize_lift::<N, H, R, _>(wrapper)) }

    fn zipmap<Extra, M>(self, mapper: M) -> Self::With<ShapeLift<Local, N, H, R, N, H, (R, Extra)>>
    where Extra: Clone + 'static,
          M: Fn(&R) -> Extra + 'static,
    { self.then_lift(Local::zipmap_lift::<N, H, R, Extra, _>(mapper)) }

    fn map_r_bi<RNew, Fwd, Bwd>(self, forward: Fwd, backward: Bwd)
        -> Self::With<ShapeLift<Local, N, H, R, N, H, RNew>>
    where RNew: Clone + 'static,
          Fwd: Fn(&R) -> RNew + 'static,
          Bwd: Fn(&RNew) -> R + 'static,
    { self.then_lift(Local::map_r_bi_lift::<N, H, R, RNew, _, _>(forward, backward)) }

    // ── treeish-side sugars ──────────────────────────────────

    fn filter_edges<P>(self, pred: P) -> Self::With<ShapeLift<Local, N, H, R, N, H, R>>
    where P: Fn(&N) -> bool + 'static,
    { self.then_lift(Local::filter_edges_lift::<N, H, R, _>(pred)) }

    fn wrap_visit<W>(self, wrapper: W) -> Self::With<ShapeLift<Local, N, H, R, N, H, R>>
    where W: Fn(&N, &mut dyn FnMut(&N), &dyn Fn(&N, &mut dyn FnMut(&N))) + 'static,
    { self.then_lift(Local::wrap_visit_lift::<N, H, R, _>(wrapper)) }

    fn memoize_by<K, KeyFn>(self, key_fn: KeyFn)
        -> Self::With<ShapeLift<Local, N, H, R, N, H, R>>
    where K: Eq + std::hash::Hash + 'static,
          KeyFn: Fn(&N) -> K + 'static,
    { self.then_lift(Local::memoize_by_lift::<N, H, R, K, _>(key_fn)) }

    // N-change is provided by the Stage-1 sugar trait
    // (`SeedSugarsLocal::map_node_bi` / `TreeishSugarsLocal::map_node_bi`)
    // since it's a reshape primitive. On `LiftedPipeline`, use
    // `.then_lift(Local::map_n_bi_lift(co, contra))` explicitly.

    // ── explainer ────────────────────────────────────────────

    fn explain(self) -> Self::With<ShapeLift<Local, N, H, R, N,
                                   ExplainerHeap<N, H, ExplainerResult<N, H, R>>,
                                   ExplainerResult<N, H, R>>>
    { self.then_lift(Local::explainer_lift::<N, H, R>()) }
}

// SeedPipeline<Local, …> no longer auto-lifts into LiftedSugarsLocal
// (Option B). LiftedSeedPipeline is Shared-pinned for now; Local
// SeedPipelines can still reach .run_from_node via TreeishSource.

// ── Impl 1: TreeishPipeline — auto-lifts first ─────────────────

impl<N, H, R> LiftedSugarsLocal<N, H, R> for TreeishPipeline<Local, N, H, R>
where N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
{
    type With<L2> = LiftedPipeline<Self, ComposedLift<IdentityLift, L2>>
    where L2: Lift<Local, N, H, R>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
          Local:    Domain<L2::N2>;

    fn then_lift<L2>(self, l: L2) -> Self::With<L2>
    where L2: Lift<Local, N, H, R>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
          Local:    Domain<L2::N2>,
    {
        LiftedPipeline::then_lift(self.lift(), l)
    }
}

// ── Impl 2: LiftedPipeline — compose at the tip ────────────────

impl<Base, L> LiftedSugarsLocal<L::N2, L::MapH, L::MapR> for LiftedPipeline<Base, L>
where Base: TreeishSource<Domain = Local>,
      Local: Domain<L::N2>,
      L: Lift<Local, Base::N, Base::H, Base::R>,
      L::N2:   Clone + 'static,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    type With<L2> = LiftedPipeline<Base, ComposedLift<L, L2>>
    where L2: Lift<Local, L::N2, L::MapH, L::MapR>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
          Local:    Domain<L2::N2>;

    fn then_lift<L2>(self, l: L2) -> Self::With<L2>
    where L2: Lift<Local, L::N2, L::MapH, L::MapR>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
          Local:    Domain<L2::N2>,
    {
        LiftedPipeline::then_lift(self, l)
    }
}
