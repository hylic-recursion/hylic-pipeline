#![allow(clippy::type_complexity)] // load-bearing: each projection is a type-level proof
//! `WrapShared` — Shared-domain build methods on `Wrap` impls.
//!
//! Each method constructs a `ShapeLift<Shared, Self::Of<UN>, …>`
//! from user inputs typed at `&UN`. `Identity` impls are
//! pass-through wrappers around `Shared::xxx_lift`. `SeedWrap`
//! impls peel `SeedNode::Node(_)` to forward to the user's closure
//! and pass `SeedNode::EntryRoot` through to the chain's `orig`
//! continuation.
//!
//! The bodies of `SeedWrap`'s methods are the closure adapters
//! that previously lived in the per-Base seed sugars file,
//! reorganised to be the single source of truth for seed-rooted
//! Stage-2 dispatch.

use std::sync::Arc;

use hylic::domain::Shared;
use hylic::domain::shared::fold::Fold;
use hylic::ops::{ShapeLift, SeedNode};
use hylic::ops::seed_node_internal::{self as sn_int, SeedNodeInner};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use super::wrap::{Identity, SeedWrap, Wrap};

// ── WrapShared trait ───────────────────────────────────────────

/// Shared-domain build methods. Each one produces a `ShapeLift`
/// over the wrapped chain N, given user closures typed at `&UN`.
/// Implemented by every `Wrap` (currently `Identity` + `SeedWrap`).
#[allow(missing_docs)] // method docs live on the trait — bodies are mechanical
pub trait WrapShared: Wrap {
    fn build_wrap_init<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + Send + Sync + 'static;

    fn build_wrap_accumulate<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + Send + Sync + 'static;

    fn build_wrap_finalize<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + Send + Sync + 'static;

    fn build_zipmap<UN, H, R, Extra, M>(m: M)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, (R, Extra)>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Extra: Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        M: Fn(&R) -> Extra + Send + Sync + 'static;

    fn build_map_r_bi<UN, H, R, RNew, Fwd, Bwd>(fwd: Fwd, bwd: Bwd)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, RNew>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        RNew: Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        Fwd: Fn(&R) -> RNew + Send + Sync + 'static,
        Bwd: Fn(&RNew) -> R + Send + Sync + 'static;

    fn build_filter_edges<UN, H, R, P>(pred: P)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        P: Fn(&UN) -> bool + Send + Sync + 'static;

    fn build_memoize_by<UN, H, R, K, KeyFn>(key_fn: KeyFn)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        K:  Eq + std::hash::Hash + Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        KeyFn: Fn(&UN) -> K + Send + Sync + 'static;

    fn build_map_n_bi<UN, UN2, H, R, Co, Contra>(co: Co, contra: Contra)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN2>, H, R>
    where
        UN:  Clone + Send + Sync + 'static,
        UN2: Clone + Send + Sync + 'static,
        H:   Clone + Send + Sync + 'static,
        R:   Clone + Send + Sync + 'static,
        Self::Of<UN>:  Clone + Send + Sync + 'static,
        Self::Of<UN2>: Clone + Send + Sync + 'static,
        Co:     Fn(&UN)  -> UN2 + Clone + Send + Sync + 'static,
        Contra: Fn(&UN2) -> UN  + Clone + Send + Sync + 'static;

    fn build_wrap_visit<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN)))
           + Send + Sync + 'static;

    fn build_explain<UN, H, R>()
        -> ShapeLift<Shared, Self::Of<UN>, H, R,
                     Self::Of<UN>,
                     ExplainerHeap<Self::Of<UN>, H, ExplainerResult<Self::Of<UN>, H, R>>,
                     ExplainerResult<Self::Of<UN>, H, R>>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static;

    fn build_explain_describe<UN, H, R, FmtFold, Emit>(fmt_ctor: FmtFold, emit: Emit)
        -> ShapeLift<Shared, Self::Of<UN>, H, R,
                     Self::Of<UN>,
                     ExplainerHeap<Self::Of<UN>, H, R>,
                     R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Self::Of<UN>: Clone + Send + Sync + 'static,
        FmtFold: Fn() -> Fold<ExplainerHeap<Self::Of<UN>, H, R>, String, String>
                 + Send + Sync + 'static,
        Emit: Fn(&str) + Send + Sync + 'static;
}

// ── Identity impl: pass-through ────────────────────────────────

impl WrapShared for Identity {
    fn build_wrap_init<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, UN, H, R, UN, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + Send + Sync + 'static,
    {
        Shared::wrap_init_lift::<UN, H, R, _>(w)
    }

    fn build_wrap_accumulate<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, UN, H, R, UN, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + Send + Sync + 'static,
    {
        Shared::wrap_accumulate_lift::<UN, H, R, _>(w)
    }

    fn build_wrap_finalize<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, UN, H, R, UN, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + Send + Sync + 'static,
    {
        Shared::wrap_finalize_lift::<UN, H, R, _>(w)
    }

    fn build_zipmap<UN, H, R, Extra, M>(m: M)
        -> ShapeLift<Shared, UN, H, R, UN, H, (R, Extra)>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Extra: Clone + Send + Sync + 'static,
        M: Fn(&R) -> Extra + Send + Sync + 'static,
    {
        Shared::zipmap_lift::<UN, H, R, Extra, _>(m)
    }

    fn build_map_r_bi<UN, H, R, RNew, Fwd, Bwd>(fwd: Fwd, bwd: Bwd)
        -> ShapeLift<Shared, UN, H, R, UN, H, RNew>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        RNew: Clone + Send + Sync + 'static,
        Fwd: Fn(&R) -> RNew + Send + Sync + 'static,
        Bwd: Fn(&RNew) -> R + Send + Sync + 'static,
    {
        Shared::map_r_bi_lift::<UN, H, R, RNew, _, _>(fwd, bwd)
    }

    fn build_filter_edges<UN, H, R, P>(pred: P)
        -> ShapeLift<Shared, UN, H, R, UN, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        P: Fn(&UN) -> bool + Send + Sync + 'static,
    {
        Shared::filter_edges_lift::<UN, H, R, _>(pred)
    }

    fn build_memoize_by<UN, H, R, K, KeyFn>(key_fn: KeyFn)
        -> ShapeLift<Shared, UN, H, R, UN, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        K:  Eq + std::hash::Hash + Clone + Send + Sync + 'static,
        KeyFn: Fn(&UN) -> K + Send + Sync + 'static,
    {
        Shared::memoize_by_lift::<UN, H, R, K, _>(key_fn)
    }

    fn build_map_n_bi<UN, UN2, H, R, Co, Contra>(co: Co, contra: Contra)
        -> ShapeLift<Shared, UN, H, R, UN2, H, R>
    where
        UN:  Clone + Send + Sync + 'static,
        UN2: Clone + Send + Sync + 'static,
        H:   Clone + Send + Sync + 'static,
        R:   Clone + Send + Sync + 'static,
        Co:     Fn(&UN)  -> UN2 + Clone + Send + Sync + 'static,
        Contra: Fn(&UN2) -> UN  + Clone + Send + Sync + 'static,
    {
        Shared::map_n_bi_lift::<UN, H, R, UN2, _, _>(co, contra)
    }

    fn build_wrap_visit<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, UN, H, R, UN, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN)))
           + Send + Sync + 'static,
    {
        Shared::wrap_visit_lift::<UN, H, R, _>(w)
    }

    fn build_explain<UN, H, R>()
        -> ShapeLift<Shared, UN, H, R,
                     UN,
                     ExplainerHeap<UN, H, ExplainerResult<UN, H, R>>,
                     ExplainerResult<UN, H, R>>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
    {
        Shared::explainer_lift::<UN, H, R>()
    }

    fn build_explain_describe<UN, H, R, FmtFold, Emit>(fmt_ctor: FmtFold, emit: Emit)
        -> ShapeLift<Shared, UN, H, R,
                     UN,
                     ExplainerHeap<UN, H, R>,
                     R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        FmtFold: Fn() -> Fold<ExplainerHeap<UN, H, R>, String, String>
                 + Send + Sync + 'static,
        Emit: Fn(&str) + Send + Sync + 'static,
    {
        Shared::explainer_describe_lift::<UN, H, R, _, _>(fmt_ctor, emit)
    }
}

// ── SeedWrap impl: peel SeedNode::Node, pass EntryRoot through ─

impl WrapShared for SeedWrap {
    fn build_wrap_init<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + Send + Sync + 'static,
    {
        let user = Arc::new(w);
        let lifted = move |ln: &SeedNode<UN>,
                           orig: &dyn Fn(&SeedNode<UN>) -> H| -> H
        {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => {
                    let user = user.clone();
                    user(n, &|inner: &UN| orig(&sn_int::node(inner.clone())))
                }
                SeedNodeInner::EntryRoot => orig(ln),
            }
        };
        Shared::wrap_init_lift::<SeedNode<UN>, H, R, _>(lifted)
    }

    fn build_wrap_accumulate<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + Send + Sync + 'static,
    {
        // No N in signature — uniform application; same as Identity.
        Shared::wrap_accumulate_lift::<SeedNode<UN>, H, R, _>(w)
    }

    fn build_wrap_finalize<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + Send + Sync + 'static,
    {
        Shared::wrap_finalize_lift::<SeedNode<UN>, H, R, _>(w)
    }

    fn build_zipmap<UN, H, R, Extra, M>(m: M)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, (R, Extra)>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        Extra: Clone + Send + Sync + 'static,
        M: Fn(&R) -> Extra + Send + Sync + 'static,
    {
        Shared::zipmap_lift::<SeedNode<UN>, H, R, Extra, _>(m)
    }

    fn build_map_r_bi<UN, H, R, RNew, Fwd, Bwd>(fwd: Fwd, bwd: Bwd)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, RNew>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        RNew: Clone + Send + Sync + 'static,
        Fwd: Fn(&R) -> RNew + Send + Sync + 'static,
        Bwd: Fn(&RNew) -> R + Send + Sync + 'static,
    {
        Shared::map_r_bi_lift::<SeedNode<UN>, H, R, RNew, _, _>(fwd, bwd)
    }

    fn build_filter_edges<UN, H, R, P>(pred: P)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        P: Fn(&UN) -> bool + Send + Sync + 'static,
    {
        let p = Arc::new(pred);
        // EntryRoot always admits its children (the entry-seed fan-out
        // is structural, not user-filterable).
        let lifted = move |ln: &SeedNode<UN>| -> bool {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => (p)(n),
                SeedNodeInner::EntryRoot => true,
            }
        };
        Shared::filter_edges_lift::<SeedNode<UN>, H, R, _>(lifted)
    }

    fn build_memoize_by<UN, H, R, K, KeyFn>(key_fn: KeyFn)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        K:  Eq + std::hash::Hash + Clone + Send + Sync + 'static,
        KeyFn: Fn(&UN) -> K + Send + Sync + 'static,
    {
        let key = Arc::new(key_fn);
        // EntryRoot is uncached (key None) — distinct per run.
        let lifted = move |ln: &SeedNode<UN>| -> Option<K> {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => Some((key)(n)),
                SeedNodeInner::EntryRoot => None,
            }
        };
        Shared::memoize_by_lift::<SeedNode<UN>, H, R, Option<K>, _>(lifted)
    }

    fn build_map_n_bi<UN, UN2, H, R, Co, Contra>(co: Co, contra: Contra)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN2>, H, R>
    where
        UN:  Clone + Send + Sync + 'static,
        UN2: Clone + Send + Sync + 'static,
        H:   Clone + Send + Sync + 'static,
        R:   Clone + Send + Sync + 'static,
        Co:     Fn(&UN)  -> UN2 + Send + Sync + 'static,
        Contra: Fn(&UN2) -> UN  + Send + Sync + 'static,
    {
        let co_arc     = Arc::new(co);
        let contra_arc = Arc::new(contra);
        let lifted_co = {
            let c = co_arc.clone();
            move |ln: &SeedNode<UN>| -> SeedNode<UN2> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot => sn_int::entry_root(),
                }
            }
        };
        let lifted_contra = {
            let c = contra_arc.clone();
            move |ln: &SeedNode<UN2>| -> SeedNode<UN> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot => sn_int::entry_root(),
                }
            }
        };
        Shared::map_n_bi_lift::<SeedNode<UN>, H, R, SeedNode<UN2>, _, _>(lifted_co, lifted_contra)
    }

    fn build_wrap_visit<UN, H, R, W>(w: W)
        -> ShapeLift<Shared, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN)))
           + Send + Sync + 'static,
    {
        let user = Arc::new(w);
        let lifted = move |ln: &SeedNode<UN>,
                           cb: &mut dyn FnMut(&SeedNode<UN>),
                           orig: &dyn Fn(&SeedNode<UN>, &mut dyn FnMut(&SeedNode<UN>))| {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => {
                    let user = user.clone();
                    let mut wrap_cb = |un: &UN| cb(&sn_int::node(un.clone()));
                    let wrap_orig = |un: &UN, inner_cb: &mut dyn FnMut(&UN)| {
                        let mut wrapped =
                            |sn: &SeedNode<UN>| if let Some(inner) = sn.as_node() { inner_cb(inner); };
                        orig(&sn_int::node(un.clone()), &mut wrapped);
                    };
                    user(n, &mut wrap_cb, &wrap_orig);
                }
                SeedNodeInner::EntryRoot => orig(ln, cb),
            }
        };
        Shared::wrap_visit_lift::<SeedNode<UN>, H, R, _>(lifted)
    }

    fn build_explain<UN, H, R>()
        -> ShapeLift<Shared, SeedNode<UN>, H, R,
                     SeedNode<UN>,
                     ExplainerHeap<SeedNode<UN>, H, ExplainerResult<SeedNode<UN>, H, R>>,
                     ExplainerResult<SeedNode<UN>, H, R>>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
    {
        Shared::explainer_lift::<SeedNode<UN>, H, R>()
    }

    fn build_explain_describe<UN, H, R, FmtFold, Emit>(fmt_ctor: FmtFold, emit: Emit)
        -> ShapeLift<Shared, SeedNode<UN>, H, R,
                     SeedNode<UN>,
                     ExplainerHeap<SeedNode<UN>, H, R>,
                     R>
    where
        UN: Clone + Send + Sync + 'static,
        H:  Clone + Send + Sync + 'static,
        R:  Clone + Send + Sync + 'static,
        FmtFold: Fn() -> Fold<ExplainerHeap<SeedNode<UN>, H, R>, String, String>
                 + Send + Sync + 'static,
        Emit: Fn(&str) + Send + Sync + 'static,
    {
        Shared::explainer_describe_lift::<SeedNode<UN>, H, R, _, _>(fmt_ctor, emit)
    }
}
