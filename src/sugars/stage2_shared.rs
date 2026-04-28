//! Stage-2 Shared sugars — the **unified** surface.
//!
//! `Stage2SugarsShared<UN, H, R>` is implemented for every
//! `Stage2Pipeline<Base, L>` with `Base: Stage2Base`. It dispatches
//! through `<<Base as Stage2Base>::Wrap as WrapShared>::build_*` for
//! the actual lift construction; sugar bodies are one-line forwarders.
//!
//! User closures are written over `&UN` (the user-facing N).
//! `Identity`-wrapped chains (treeish-rooted) see closures applied
//! directly; `SeedWrap`-wrapped chains (seed-rooted) see closures
//! peeled from `SeedNode::Node(_)` with `EntryRoot` passed through to
//! the chain's `orig` continuation.
//!
//! Bounds at consumption — `then_lift` is unconstrained at the
//! pipeline-struct level (cf. `stage2/primitives.rs`); each sugar
//! method's bound is the minimum needed to type its build call.

#![allow(missing_docs)] // surface mirrors documented primitive constructors
#![allow(clippy::type_complexity)] // load-bearing: each projection is a type-level proof

use hylic::domain::{Domain, Shared};
use hylic::domain::shared::fold::Fold;
use hylic::ops::{ComposedLift, IdentityLift, Lift, ShapeLift};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use crate::source::TreeishSource;
use crate::treeish::TreeishPipeline;
use crate::stage2::{Stage2Pipeline, Stage2Base, Wrap};
use crate::stage2::WrapShared;

// ── Trait ──────────────────────────────────────────────────────

pub trait Stage2SugarsShared<UN, H, R>: Sized
where
    UN: Clone + Send + Sync + 'static,
    H:  Clone + Send + Sync + 'static,
    R:  Clone + Send + Sync + 'static,
{
    type Base: Stage2Base;
    type With<L2>;

    /// Sole primitive: append a lift to the chain. Unconstrained on
    /// the trait surface — chain validity is enforced at consumption
    /// (`.run`), in the spirit of `Stage2Pipeline::then_lift`.
    fn then_lift<L2>(self, l: L2) -> Self::With<L2>;

    // ── fold-side sugars ─────────────────────────────────────

    // ANCHOR: stage2_sugars_wrap_init
    fn wrap_init<W>(self, w: W) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_wrap_init::<UN, H, R, _>(w))
    }
    // ANCHOR_END: stage2_sugars_wrap_init

    fn wrap_accumulate<W>(self, w: W) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_wrap_accumulate::<UN, H, R, _>(w))
    }

    fn wrap_finalize<W>(self, w: W) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_wrap_finalize::<UN, H, R, _>(w))
    }

    fn zipmap<Extra, M>(self, m: M) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, (R, Extra)>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        Extra: Clone + Send + Sync + 'static,
        M: Fn(&R) -> Extra + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_zipmap::<UN, H, R, Extra, _>(m))
    }

    fn map_r_bi<RNew, Fwd, Bwd>(self, fwd: Fwd, bwd: Bwd) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, RNew>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        RNew: Clone + Send + Sync + 'static,
        Fwd: Fn(&R) -> RNew + Send + Sync + 'static,
        Bwd: Fn(&RNew) -> R + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_map_r_bi::<UN, H, R, RNew, _, _>(fwd, bwd))
    }

    // ── treeish-side sugars ──────────────────────────────────

    fn filter_edges<P>(self, pred: P) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        P: Fn(&UN) -> bool + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_filter_edges::<UN, H, R, _>(pred))
    }

    fn memoize_by<K, KeyFn>(self, key_fn: KeyFn) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
        KeyFn: Fn(&UN) -> K + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_memoize_by::<UN, H, R, K, _>(key_fn))
    }

    fn wrap_visit<W>(self, w: W) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN)))
           + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_wrap_visit::<UN, H, R, _>(w))
    }

    // ── N-change ─────────────────────────────────────────────

    fn map_n_bi<UN2, Co, Contra>(self, co: Co, contra: Contra) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN2>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>:  Clone + Send + Sync + 'static,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN2>: Clone + Send + Sync + 'static,
        UN2: Clone + Send + Sync + 'static,
        Co:     Fn(&UN)  -> UN2 + Clone + Send + Sync + 'static,
        Contra: Fn(&UN2) -> UN  + Clone + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_map_n_bi::<UN, UN2, H, R, _, _>(co, contra))
    }

    // ── explainer ────────────────────────────────────────────

    fn explain(self) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>,
        ExplainerHeap<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H,
                      ExplainerResult<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>,
        ExplainerResult<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_explain::<UN, H, R>())
    }

    fn explain_describe<FmtFold, Emit>(self, fmt_ctor: FmtFold, emit: Emit) -> Self::With<ShapeLift<Shared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>,
        ExplainerHeap<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>,
        R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapShared,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + Send + Sync + 'static,
        FmtFold: Fn() -> Fold<ExplainerHeap<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>, String, String>
                 + Send + Sync + 'static,
        Emit: Fn(&str) + Send + Sync + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapShared>::build_explain_describe::<UN, H, R, _, _>(fmt_ctor, emit))
    }
}

// ── Auto-lift on TreeishPipeline ───────────────────────────────
//
// Stage-1 `TreeishPipeline<Shared, …>` implements the trait by
// auto-lifting via `.lift()` first, then composing the new lift on
// top.

impl<N, H, R> Stage2SugarsShared<N, H, R> for TreeishPipeline<Shared, N, H, R>
where
    N: Clone + Send + Sync + 'static,
    H: Clone + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
    <Shared as Domain<N>>::Graph<N>:   Clone,
    <Shared as Domain<N>>::Fold<H, R>: Clone,
{
    type Base = Self;
    type With<L2> = Stage2Pipeline<Self, ComposedLift<IdentityLift, L2>>;

    fn then_lift<L2>(self, l: L2) -> Self::With<L2> {
        self.lift().then_lift(l)
    }
}

// ── Stage2Pipeline blanket impl ────────────────────────────────
//
// Single canonical impl covering both treeish-rooted and seed-rooted
// chains. Wrap dispatch happens inside the build methods of the
// `WrapShared` impl on `Base::Wrap`.

// Blanket impl bounding the trait params (UN, H, R) to the chain's
// tip via equality on `L::N2`/`L::MapH`/`L::MapR`. UN is the
// user-facing N at the tip — Rust unifies it through the Wrap GAT
// (`L::N2 = <Wrap::Of<UN>>`). The chain's INPUT type is
// `<Base::Wrap as Wrap>::Of<Base::UserN>`: equal to `Base::N` on
// `Identity`-wrapped (treeish-rooted) chains, `SeedNode<Base::N>` on
// `SeedWrap`-wrapped (seed-rooted) chains.
impl<Base, L, UN, H, R> Stage2SugarsShared<UN, H, R> for Stage2Pipeline<Base, L>
where
    Base: Stage2Base + TreeishSource<Domain = Shared>,
    Base::H: Send + Sync,
    Base::R: Send + Sync,
    Base::UserN: Clone + Send + Sync + 'static,
    UN: Clone + Send + Sync + 'static,
    H:  Clone + Send + Sync + 'static,
    R:  Clone + Send + Sync + 'static,
    <Base::Wrap as Wrap>::Of<Base::UserN>: Clone + Send + Sync + 'static,
    <Base::Wrap as Wrap>::Of<UN>:          Clone + Send + Sync + 'static,
    Shared: Domain<<Base::Wrap as Wrap>::Of<Base::UserN>>,
    Shared: Domain<<Base::Wrap as Wrap>::Of<UN>>,
    L: Lift<Shared,
            <Base::Wrap as Wrap>::Of<Base::UserN>,
            Base::H, Base::R,
            N2   = <Base::Wrap as Wrap>::Of<UN>,
            MapH = H,
            MapR = R>,
{
    type Base = Base;
    type With<L2> = Stage2Pipeline<Base, ComposedLift<L, L2>>;

    fn then_lift<L2>(self, l: L2) -> Self::With<L2> {
        Stage2Pipeline::then_lift(self, l)
    }
}
