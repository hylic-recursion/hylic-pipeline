#![allow(clippy::type_complexity)] // load-bearing: each projection is a type-level proof
//! `WrapLocal` — Local-domain build methods on `Wrap` impls.
//!
//! Mirror of `wrap_shared.rs` with `Rc` storage and no `Send + Sync`
//! bounds. Does not include `build_explain_describe` because
//! `Local::explainer_describe_lift` does not exist.

use std::rc::Rc;

use hylic::domain::Local;
use hylic::ops::{ShapeLift, SeedNode};
use hylic::ops::seed_node_internal::{self as sn_int, SeedNodeInner};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use super::wrap::{Identity, SeedWrap, Wrap};

// ── WrapLocal trait ────────────────────────────────────────────

/// Local-domain build methods on `Wrap`. Closures need not be
/// `Send + Sync`; storage is `Rc`-based.
#[allow(missing_docs)]
pub trait WrapLocal: Wrap {
    fn build_wrap_init<UN, H, R, W>(w: W)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + 'static;

    fn build_wrap_accumulate<UN, H, R, W>(w: W)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + 'static;

    fn build_wrap_finalize<UN, H, R, W>(w: W)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + 'static;

    fn build_zipmap<UN, H, R, Extra, M>(m: M)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, (R, Extra)>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Extra: Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        M: Fn(&R) -> Extra + 'static;

    fn build_map_r_bi<UN, H, R, RNew, Fwd, Bwd>(fwd: Fwd, bwd: Bwd)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, RNew>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        RNew: Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        Fwd: Fn(&R) -> RNew + 'static,
        Bwd: Fn(&RNew) -> R + 'static;

    fn build_filter_edges<UN, H, R, P>(pred: P)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        P: Fn(&UN) -> bool + 'static;

    fn build_memoize_by<UN, H, R, K, KeyFn>(key_fn: KeyFn)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        K:  Eq + std::hash::Hash + Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        KeyFn: Fn(&UN) -> K + 'static;

    fn build_map_n_bi<UN, UN2, H, R, Co, Contra>(co: Co, contra: Contra)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN2>, H, R>
    where
        UN:  Clone + 'static,
        UN2: Clone + 'static,
        H:   Clone + 'static,
        R:   Clone + 'static,
        Self::Of<UN>:  Clone + 'static,
        Self::Of<UN2>: Clone + 'static,
        Co:     Fn(&UN)  -> UN2 + Clone + 'static,
        Contra: Fn(&UN2) -> UN  + Clone + 'static;

    fn build_wrap_visit<UN, H, R, W>(w: W)
        -> ShapeLift<Local, Self::Of<UN>, H, R, Self::Of<UN>, H, R>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Self::Of<UN>: Clone + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN))) + 'static;

    fn build_explain<UN, H, R>()
        -> ShapeLift<Local, Self::Of<UN>, H, R,
                     Self::Of<UN>,
                     ExplainerHeap<Self::Of<UN>, H, ExplainerResult<Self::Of<UN>, H, R>>,
                     ExplainerResult<Self::Of<UN>, H, R>>
    where
        UN: Clone + 'static,
        H:  Clone + 'static,
        R:  Clone + 'static,
        Self::Of<UN>: Clone + 'static;
}

// ── Identity impl ──────────────────────────────────────────────

impl WrapLocal for Identity {
    fn build_wrap_init<UN, H, R, W>(w: W)
        -> ShapeLift<Local, UN, H, R, UN, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + 'static,
    {
        Local::wrap_init_lift::<UN, H, R, _>(w)
    }

    fn build_wrap_accumulate<UN, H, R, W>(w: W)
        -> ShapeLift<Local, UN, H, R, UN, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + 'static,
    {
        Local::wrap_accumulate_lift::<UN, H, R, _>(w)
    }

    fn build_wrap_finalize<UN, H, R, W>(w: W)
        -> ShapeLift<Local, UN, H, R, UN, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + 'static,
    {
        Local::wrap_finalize_lift::<UN, H, R, _>(w)
    }

    fn build_zipmap<UN, H, R, Extra, M>(m: M)
        -> ShapeLift<Local, UN, H, R, UN, H, (R, Extra)>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        Extra: Clone + 'static,
        M: Fn(&R) -> Extra + 'static,
    {
        Local::zipmap_lift::<UN, H, R, Extra, _>(m)
    }

    fn build_map_r_bi<UN, H, R, RNew, Fwd, Bwd>(fwd: Fwd, bwd: Bwd)
        -> ShapeLift<Local, UN, H, R, UN, H, RNew>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        RNew: Clone + 'static,
        Fwd: Fn(&R) -> RNew + 'static,
        Bwd: Fn(&RNew) -> R + 'static,
    {
        Local::map_r_bi_lift::<UN, H, R, RNew, _, _>(fwd, bwd)
    }

    fn build_filter_edges<UN, H, R, P>(pred: P)
        -> ShapeLift<Local, UN, H, R, UN, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        P: Fn(&UN) -> bool + 'static,
    {
        Local::filter_edges_lift::<UN, H, R, _>(pred)
    }

    fn build_memoize_by<UN, H, R, K, KeyFn>(key_fn: KeyFn)
        -> ShapeLift<Local, UN, H, R, UN, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        K:  Eq + std::hash::Hash + Clone + 'static,
        KeyFn: Fn(&UN) -> K + 'static,
    {
        Local::memoize_by_lift::<UN, H, R, K, _>(key_fn)
    }

    fn build_map_n_bi<UN, UN2, H, R, Co, Contra>(co: Co, contra: Contra)
        -> ShapeLift<Local, UN, H, R, UN2, H, R>
    where
        UN:  Clone + 'static, UN2: Clone + 'static,
        H:   Clone + 'static, R:   Clone + 'static,
        Co:     Fn(&UN)  -> UN2 + Clone + 'static,
        Contra: Fn(&UN2) -> UN  + Clone + 'static,
    {
        Local::map_n_bi_lift::<UN, H, R, UN2, _, _>(co, contra)
    }

    fn build_wrap_visit<UN, H, R, W>(w: W)
        -> ShapeLift<Local, UN, H, R, UN, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN))) + 'static,
    {
        Local::wrap_visit_lift::<UN, H, R, _>(w)
    }

    fn build_explain<UN, H, R>()
        -> ShapeLift<Local, UN, H, R,
                     UN,
                     ExplainerHeap<UN, H, ExplainerResult<UN, H, R>>,
                     ExplainerResult<UN, H, R>>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
    {
        Local::explainer_lift::<UN, H, R>()
    }
}

// ── SeedWrap impl ──────────────────────────────────────────────

impl WrapLocal for SeedWrap {
    fn build_wrap_init<UN, H, R, W>(w: W)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&UN, &dyn Fn(&UN) -> H) -> H + 'static,
    {
        let user = Rc::new(w);
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
        Local::wrap_init_lift::<SeedNode<UN>, H, R, _>(lifted)
    }

    fn build_wrap_accumulate<UN, H, R, W>(w: W)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&mut H, &R, &dyn Fn(&mut H, &R)) + 'static,
    {
        Local::wrap_accumulate_lift::<SeedNode<UN>, H, R, _>(w)
    }

    fn build_wrap_finalize<UN, H, R, W>(w: W)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&H, &dyn Fn(&H) -> R) -> R + 'static,
    {
        Local::wrap_finalize_lift::<SeedNode<UN>, H, R, _>(w)
    }

    fn build_zipmap<UN, H, R, Extra, M>(m: M)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, (R, Extra)>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        Extra: Clone + 'static,
        M: Fn(&R) -> Extra + 'static,
    {
        Local::zipmap_lift::<SeedNode<UN>, H, R, Extra, _>(m)
    }

    fn build_map_r_bi<UN, H, R, RNew, Fwd, Bwd>(fwd: Fwd, bwd: Bwd)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, RNew>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        RNew: Clone + 'static,
        Fwd: Fn(&R) -> RNew + 'static,
        Bwd: Fn(&RNew) -> R + 'static,
    {
        Local::map_r_bi_lift::<SeedNode<UN>, H, R, RNew, _, _>(fwd, bwd)
    }

    fn build_filter_edges<UN, H, R, P>(pred: P)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        P: Fn(&UN) -> bool + 'static,
    {
        let p = Rc::new(pred);
        let lifted = move |ln: &SeedNode<UN>| -> bool {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => (p)(n),
                SeedNodeInner::EntryRoot => true,
            }
        };
        Local::filter_edges_lift::<SeedNode<UN>, H, R, _>(lifted)
    }

    fn build_memoize_by<UN, H, R, K, KeyFn>(key_fn: KeyFn)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        K:  Eq + std::hash::Hash + Clone + 'static,
        KeyFn: Fn(&UN) -> K + 'static,
    {
        let key = Rc::new(key_fn);
        let lifted = move |ln: &SeedNode<UN>| -> Option<K> {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => Some((key)(n)),
                SeedNodeInner::EntryRoot => None,
            }
        };
        Local::memoize_by_lift::<SeedNode<UN>, H, R, Option<K>, _>(lifted)
    }

    fn build_map_n_bi<UN, UN2, H, R, Co, Contra>(co: Co, contra: Contra)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN2>, H, R>
    where
        UN:  Clone + 'static, UN2: Clone + 'static,
        H:   Clone + 'static, R:   Clone + 'static,
        Co:     Fn(&UN)  -> UN2 + 'static,
        Contra: Fn(&UN2) -> UN  + 'static,
    {
        let co_rc     = Rc::new(co);
        let contra_rc = Rc::new(contra);
        let lifted_co = {
            let c = co_rc.clone();
            move |ln: &SeedNode<UN>| -> SeedNode<UN2> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot => sn_int::entry_root(),
                }
            }
        };
        let lifted_contra = {
            let c = contra_rc.clone();
            move |ln: &SeedNode<UN2>| -> SeedNode<UN> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot => sn_int::entry_root(),
                }
            }
        };
        Local::map_n_bi_lift::<SeedNode<UN>, H, R, SeedNode<UN2>, _, _>(lifted_co, lifted_contra)
    }

    fn build_wrap_visit<UN, H, R, W>(w: W)
        -> ShapeLift<Local, SeedNode<UN>, H, R, SeedNode<UN>, H, R>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
        W: Fn(&UN, &mut dyn FnMut(&UN), &dyn Fn(&UN, &mut dyn FnMut(&UN))) + 'static,
    {
        let user = Rc::new(w);
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
        Local::wrap_visit_lift::<SeedNode<UN>, H, R, _>(lifted)
    }

    fn build_explain<UN, H, R>()
        -> ShapeLift<Local, SeedNode<UN>, H, R,
                     SeedNode<UN>,
                     ExplainerHeap<SeedNode<UN>, H, ExplainerResult<SeedNode<UN>, H, R>>,
                     ExplainerResult<SeedNode<UN>, H, R>>
    where
        UN: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
    {
        Local::explainer_lift::<SeedNode<UN>, H, R>()
    }
}
