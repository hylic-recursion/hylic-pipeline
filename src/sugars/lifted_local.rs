//! Stage-2 Local sugars — the **unified** surface (Local mirror).
//!
//! Mirror of `lifted_shared.rs` with `Rc` storage and no `Send + Sync`
//! bounds. Same Wrap-as-dispatcher pattern: each sugar method is a
//! one-line forwarder through `<<Self::Base as Stage2Base>::Wrap as
//! WrapLocal>::build_*::<…>(args)`.

#![allow(missing_docs)] // surface mirrors documented primitive constructors

use hylic::domain::{Domain, Local};
use hylic::ops::{ComposedLift, IdentityLift, ShapeLift};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use crate::source::TreeishSource;
use crate::treeish::TreeishPipeline;
use crate::stage2::{Stage2Pipeline, Stage2Base, Wrap};
use crate::stage2::wrap_local::WrapLocal;

// ── Trait ──────────────────────────────────────────────────────

pub trait Stage2SugarsLocal<UN, H, R>: Sized
where
    UN: Clone + 'static,
    H:  Clone + 'static,
    R:  Clone + 'static,
{
    type Base: Stage2Base;
    type With<L2>;

    fn then_lift<L2>(self, l: L2) -> Self::With<L2>;

    // ── fold-side sugars ─────────────────────────────────────

    fn wrap_init<W>(self, w: W) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_wrap_init::<UN, H, R, _>(w))
    }

    fn wrap_accumulate<W>(self, w: W) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_wrap_accumulate::<UN, H, R, _>(w))
    }

    fn wrap_finalize<W>(self, w: W) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_wrap_finalize::<UN, H, R, _>(w))
    }

    fn zipmap<Extra, M>(self, m: M) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, (R, Extra)>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        Extra: Clone + 'static,
        M: Fn(&R) -> Extra + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_zipmap::<UN, H, R, Extra, _>(m))
    }

    fn map_r_bi<RNew, Fwd, Bwd>(self, fwd: Fwd, bwd: Bwd) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, RNew>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        RNew: Clone + 'static,
        Fwd: Fn(&R) -> RNew + 'static,
        Bwd: Fn(&RNew) -> R + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_map_r_bi::<UN, H, R, RNew, _, _>(fwd, bwd))
    }

    // ── treeish-side sugars ──────────────────────────────────

    fn filter_edges<P>(self, pred: P) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        P: Fn(&UN) -> bool + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_filter_edges::<UN, H, R, _>(pred))
    }

    fn memoize_by<K, KeyFn>(self, key_fn: KeyFn) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        K: Eq + std::hash::Hash + Clone + 'static,
        KeyFn: Fn(&UN) -> K + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_memoize_by::<UN, H, R, K, _>(key_fn))
    }

    fn wrap_visit<W>(self, w: W) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN))) + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_wrap_visit::<UN, H, R, _>(w))
    }

    // ── N-change ─────────────────────────────────────────────

    fn map_n_bi<UN2, Co, Contra>(self, co: Co, contra: Contra) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN2>, H, R>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>:  Clone + 'static,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN2>: Clone + 'static,
        UN2: Clone + 'static,
        Co:     Fn(&UN)  -> UN2 + Clone + 'static,
        Contra: Fn(&UN2) -> UN  + Clone + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_map_n_bi::<UN, UN2, H, R, _, _>(co, contra))
    }

    // ── explainer ────────────────────────────────────────────

    fn explain(self) -> Self::With<ShapeLift<Local,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>,
        ExplainerHeap<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H,
                      ExplainerResult<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>,
        ExplainerResult<<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>, H, R>>>
    where
        <Self::Base as Stage2Base>::Wrap: WrapLocal,
        <<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>: Clone + 'static,
    {
        self.then_lift(<<Self::Base as Stage2Base>::Wrap as WrapLocal>::build_explain::<UN, H, R>())
    }
}

// ── Auto-lift on TreeishPipeline ───────────────────────────────

impl<N, H, R> Stage2SugarsLocal<N, H, R> for TreeishPipeline<Local, N, H, R>
where
    N: Clone + 'static,
    H: Clone + 'static,
    R: Clone + 'static,
    <Local as Domain<N>>::Graph<N>:   Clone,
    <Local as Domain<N>>::Fold<H, R>: Clone,
{
    type Base = Self;
    type With<L2> = Stage2Pipeline<Self, ComposedLift<IdentityLift, L2>>;

    fn then_lift<L2>(self, l: L2) -> Self::With<L2> {
        self.lift().then_lift(l)
    }
}

// ── Stage2Pipeline blanket impl ────────────────────────────────

// Mirror of Stage2SugarsShared blanket impl with `Local` storage.
// See `lifted_shared.rs` for design notes.
impl<Base, L, UN, H, R> Stage2SugarsLocal<UN, H, R> for Stage2Pipeline<Base, L>
where
    Base: Stage2Base + TreeishSource<Domain = Local>,
    Base::UserN: Clone + 'static,
    UN: Clone + 'static,
    H:  Clone + 'static,
    R:  Clone + 'static,
    <Base::Wrap as Wrap>::Of<Base::UserN>: Clone + 'static,
    <Base::Wrap as Wrap>::Of<UN>:          Clone + 'static,
    Local: Domain<<Base::Wrap as Wrap>::Of<Base::UserN>>,
    Local: Domain<<Base::Wrap as Wrap>::Of<UN>>,
    L: hylic::ops::Lift<Local,
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
