//! Shared-domain stage-2 sugars for seed-rooted `Stage2Pipeline`s.
//!
//! Two populations of sugar:
//!
//! 1. **N-parametric library lifts** — `explain`. These instantiate
//!    the library lift at `N = SeedNode<CurN>` and post-compose.
//!    Entry is handled as a first-class node value by the lift
//!    itself — its node field naturally takes `SeedNode::Entry`.
//!
//! 2. **User-closure sugars** — `wrap_init`, `wrap_accumulate`,
//!    `wrap_finalize`, `zipmap`, `map_r_bi`, `filter_edges`,
//!    `memoize_by`. Sugars whose signature involves `&N` wrap the
//!    user's closure with Node/Entry dispatch internally (Node
//!    applies; Entry passes through). Sugars that don't mention N
//!    (wrap_accumulate, wrap_finalize, zipmap, map_r_bi) apply
//!    uniformly at both variants — no dispatch needed.

use std::sync::Arc;

use hylic::domain::{Domain, Shared};
use hylic::domain::shared::fold::Fold;
use hylic::ops::{ComposedLift, Lift, SeedNode, ShapeLift};
use hylic::ops::seed_node_internal::{self as sn_int, SeedNodeInner};
use hylic::prelude::explainer::{ExplainerHeap, ExplainerResult};

use crate::stage2::Stage2Pipeline;
use super::super::seed::SeedPipeline;

impl<N, Seed, H, R, L, CurN> Stage2Pipeline<SeedPipeline<Shared, N, Seed, H, R>, L>
where N:    Clone + Send + Sync + 'static,
      Seed: Clone + Send + Sync + 'static,
      H:    Clone + Send + Sync + 'static,
      R:    Clone + Send + Sync + 'static,
      CurN: Clone + Send + Sync + 'static,
      Shared: Domain<N> + Domain<SeedNode<N>> + Domain<SeedNode<CurN>>,
      L:    Lift<Shared, SeedNode<N>, H, R, N2 = SeedNode<CurN>>,
      L::MapH: Clone + Send + Sync + 'static,
      L::MapR: Clone + Send + Sync + 'static,
{
    // ── User-closure, N-aware: Node/Entry dispatch ─────────

    /// Wrap the init at every real Node. Entry's init
    /// (SeedLift's `entry_heap`) is untouched.
    pub fn wrap_init<W>(self, user_wrap: W)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&CurN, &dyn Fn(&CurN) -> L::MapH) -> L::MapH + Send + Sync + 'static,
    {
        let user = Arc::new(user_wrap);
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
        self.then_lift(Shared::wrap_init_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(lifted_w))
    }

    /// Memoise by a key derived from real Nodes. Entry bypasses the
    /// cache (`key_fn` can't apply without an N).
    pub fn memoize_by<K, KeyFn>(self, key_fn: KeyFn)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
          KeyFn: Fn(&CurN) -> K + Send + Sync + 'static,
    {
        let key = Arc::new(key_fn);
        // Cache key is Option<K>: None for Entry (distinct per run),
        // Some(k) for Node(n).
        let lifted_key = move |ln: &SeedNode<CurN>| -> Option<K> {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => Some((key)(n)),
                SeedNodeInner::EntryRoot   => None,
            }
        };
        self.then_lift(Shared::memoize_by_lift::<SeedNode<CurN>, L::MapH, L::MapR, Option<K>, _>(lifted_key))
    }

    /// Filter edges between real Nodes. Entry's fan-out (from
    /// `SeedLift`) is not filtered; the predicate is over `&CurN`.
    pub fn filter_edges<P>(self, pred: P)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where P: Fn(&CurN) -> bool + Send + Sync + 'static,
    {
        let p = Arc::new(pred);
        let lifted_p = move |ln: &SeedNode<CurN>| -> bool {
            match sn_int::inner(ln) {
                SeedNodeInner::Node(n) => (p)(n),
                SeedNodeInner::EntryRoot   => true,
            }
        };
        self.then_lift(Shared::filter_edges_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(lifted_p))
    }

    // ── N-free sugars: applied uniformly ──────────────────

    /// Wrap the accumulate closure. No N in signature → applied
    /// uniformly at Node and Entry.
    pub fn wrap_accumulate<W>(self, wrapper: W)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&mut L::MapH, &L::MapR, &dyn Fn(&mut L::MapH, &L::MapR))
            + Send + Sync + 'static,
    {
        self.then_lift(Shared::wrap_accumulate_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(wrapper))
    }

    /// Wrap the finalize closure. No N in signature → applied
    /// uniformly.
    pub fn wrap_finalize<W>(self, wrapper: W)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, L::MapR>>,
        >
    where W: Fn(&L::MapH, &dyn Fn(&L::MapH) -> L::MapR) -> L::MapR + Send + Sync + 'static,
    {
        self.then_lift(Shared::wrap_finalize_lift::<SeedNode<CurN>, L::MapH, L::MapR, _>(wrapper))
    }

    /// Pair R with an extra value per node. R-only; no N dispatch.
    pub fn zipmap<Extra, M>(self, mapper: M)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, (L::MapR, Extra)>>,
        >
    where Extra: Clone + Send + Sync + 'static,
          M: Fn(&L::MapR) -> Extra + Send + Sync + 'static,
    {
        self.then_lift(Shared::zipmap_lift::<SeedNode<CurN>, L::MapH, L::MapR, Extra, _>(mapper))
    }

    /// Bijectively transform R. R-only; no N dispatch.
    pub fn map_r_bi<RNew, Fwd, Bwd>(self, forward: Fwd, backward: Bwd)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>, L::MapH, RNew>>,
        >
    where RNew: Clone + Send + Sync + 'static,
          Fwd: Fn(&L::MapR) -> RNew + Send + Sync + 'static,
          Bwd: Fn(&RNew) -> L::MapR + Send + Sync + 'static,
    {
        self.then_lift(Shared::map_r_bi_lift::<SeedNode<CurN>, L::MapH, L::MapR, RNew, _, _>(forward, backward))
    }

    // ── N-change sugar: map_n_bi ──────────────────────────
    //
    // Stage-2 bijective N-change on a seed-closed chain. The user's
    // (co, contra) run against the base `CurN`; the wrapper inside
    // preserves Entry as Entry and maps Node(n) ↔ Node(n2).

    /// Bijective N-change at Stage 2 on a seed-closed chain.
    /// Entry is mapped to Entry identically; Node(n) is mapped
    /// via the user-supplied (co, contra).
    pub fn map_n_bi<N2, Co, Contra>(self, co: Co, contra: Contra)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<N2>, L::MapH, L::MapR>>,
        >
    where N2: Clone + Send + Sync + 'static,
          Co:     Fn(&CurN) -> N2   + Send + Sync + 'static,
          Contra: Fn(&N2)   -> CurN + Send + Sync + 'static,
          Shared: Domain<SeedNode<N2>>,
    {
        let co_arc     = Arc::new(co);
        let contra_arc = Arc::new(contra);
        let lifted_co = {
            let c = co_arc.clone();
            move |ln: &SeedNode<CurN>| -> SeedNode<N2> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot   => sn_int::entry_root(),
                }
            }
        };
        let lifted_contra = {
            let c = contra_arc.clone();
            move |ln: &SeedNode<N2>| -> SeedNode<CurN> {
                match sn_int::inner(ln) {
                    SeedNodeInner::Node(n) => sn_int::node((c)(n)),
                    SeedNodeInner::EntryRoot   => sn_int::entry_root(),
                }
            }
        };
        self.then_lift(Shared::map_n_bi_lift::<SeedNode<CurN>, L::MapH, L::MapR, SeedNode<N2>, _, _>(lifted_co, lifted_contra))
    }

    // ── N-parametric library lift: explain ─────────────────

    /// Compose the whole-tree explainer. `ExplainerHeap.node` takes
    /// `SeedNode<CurN>` — `Entry` is a first-class value; no
    /// sentinel needed.
    pub fn explain(self)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>,
                                               ExplainerHeap<SeedNode<CurN>, L::MapH,
                                                             ExplainerResult<SeedNode<CurN>, L::MapH, L::MapR>>,
                                               ExplainerResult<SeedNode<CurN>, L::MapH, L::MapR>>>,
        >
    {
        self.then_lift(Shared::explainer_lift::<SeedNode<CurN>, L::MapH, L::MapR>())
    }

    /// Compose the streaming-describe explainer. Emits a rendered
    /// trace string at each node's finalize (Entry included); R is
    /// unchanged downstream.
    pub fn explain_describe<FmtFold, Emit>(self, fmt_ctor: FmtFold, emit: Emit)
        -> Stage2Pipeline<
            SeedPipeline<Shared, N, Seed, H, R>,
            ComposedLift<L, ShapeLift<Shared, SeedNode<CurN>, L::MapH, L::MapR,
                                               SeedNode<CurN>,
                                               ExplainerHeap<SeedNode<CurN>, L::MapH, L::MapR>,
                                               L::MapR>>,
        >
    where FmtFold: Fn() -> Fold<ExplainerHeap<SeedNode<CurN>, L::MapH, L::MapR>, String, String>
                   + Send + Sync + 'static,
          Emit: Fn(&str) + Send + Sync + 'static,
    {
        self.then_lift(Shared::explainer_describe_lift::<SeedNode<CurN>, L::MapH, L::MapR, _, _>(fmt_ctor, emit))
    }
}
