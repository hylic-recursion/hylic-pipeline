//! `.run` / `.run_from_slice` for `Stage2Pipeline<Base, L>`.
//!
//! Two layers:
//!
//! 1. **Internal foundation** — `run_with_inputs` is generic over
//!    every `(Domain, Base)` cell. The base supplies a pre-chain
//!    lift and the executor's post-chain root reference via
//!    [`Stage2Base::provide_run_essentials`]; the
//!    `(treeish<N>, fold<N,H,R>)` pair is yielded through the
//!    inherited [`crate::source::TreeishSource::with_treeish`].
//!
//! 2. **Public surface** — for treeish-rooted bases, the existing
//!    `PipelineExec::run_from_node(&exec, &root)` blanket continues
//!    to provide the natural shape (the `Stage2Pipeline`
//!    `TreeishSource` impl exposes the post-chain `(treeish, fold)`
//!    pair, and `run_from_node` borrows the user's `&CurN`). For
//!    seed-rooted bases, the public `run(exec, root_seeds, entry_heap)`
//!    and `run_from_slice(exec, seeds, entry_heap)` mirror the
//!    pre-unification shape exactly — separate args, no
//!    user-visible tuple.
//!
//! `gat_helpers` remains the per-domain GAT-pinning surface, consumed
//! by the per-domain `Stage2Base` impls in
//! `seed/stage2_base_shared.rs` / `seed/stage2_base_local.rs` plus
//! the sugar build closures in `stage2/wrap/`.
//!
//! ## The `CurN` method-level parameter
//!
//! `CurN` is the user-facing N at the chain tip — i.e. the inner N
//! after any `map_n_bi` lifts. It is `<Base as TreeishSource>::N`
//! whenever the chain does not change the user N.
//!
//! Pinning `L::N2 = <Base::Wrap as Wrap>::Of<CurN>` (a where-clause
//! associated-type-equality predicate on each method) makes the
//! executor signature line up: the descend root is at the post-chain
//! type `<Wrap as Wrap>::Of<CurN>`, and the executor consumes
//! `&L::N2`. For `Identity`-Wrap that is `&CurN`; for `SeedWrap`
//! that is `&SeedNode<CurN>` (the base constructs `EntryRoot::<CurN>`
//! locally — payload-free, so `CurN` need only line up at the type
//! level).
//!
//! `CurN` is method-level rather than impl-level because Rust's
//! E0207 rules reject impl-level type params that are constrained
//! only through a deep GAT projection like
//! `L::N2 = <<Base::Wrap as Wrap>::Of<CurN>>` — the projection isn't
//! recognised as constraining the parameter directly enough.
//! Method-level lifts the constraint into a per-call where-clause,
//! which Rust accepts.

pub(crate) mod gat_helpers;

use hylic::domain::Domain;
use hylic::exec::Executor;
use hylic::ops::{Lift, TreeOps};

use crate::source::TreeishSource;
use crate::stage2::{Stage2Base, Stage2BaseSlice, Stage2Pipeline, Wrap};

// ── Internal foundation: tuple-form run, generic over Base ────

impl<Base, L> Stage2Pipeline<Base, L>
where
    Base: Stage2Base,
{
    /// Drive the chain to completion. Internal — the user-facing
    /// methods (`run`, `run_from_slice` on seed-rooted; `run_from_node`
    /// on treeish-rooted via the existing `PipelineExec` blanket)
    /// adapt the call shape.
    fn run_with_inputs<E, CurN>(&self, exec: &E, inputs: <Base as Stage2Base>::RunInputs<'_, CurN>) -> L::MapR
    where
        CurN: Clone + 'static,
        <Base as TreeishSource>::Domain: Domain<<Base as TreeishSource>::N>
            + Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>>
            + Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>,
        <Base as Stage2Base>::PreLift: Lift<
                <Base as TreeishSource>::Domain,
                <Base as TreeishSource>::N,
                <Base as TreeishSource>::H,
                <Base as TreeishSource>::R,
                N2 = <<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>,
                MapH = <Base as TreeishSource>::H,
                MapR = <Base as TreeishSource>::R,
            >,
        L: Lift<
                <Base as TreeishSource>::Domain,
                <<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>,
                <Base as TreeishSource>::H,
                <Base as TreeishSource>::R,
                N2 = <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
            >,
        L::MapH: Clone + 'static,
        L::MapR: Clone + 'static,
        E: Executor<
                <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
                L::MapR,
                <Base as TreeishSource>::Domain,
                <<Base as TreeishSource>::Domain as Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>>::Graph<
                    <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
                >,
            >,
        <<Base as TreeishSource>::Domain as Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>>::Graph<
            <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
        >: TreeOps<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>,
    {
        self.base.with_treeish(|t0, f0| {
            self.base
                .provide_run_essentials::<CurN, _>(inputs, |pre, root| {
                    pre.apply(t0, f0, |t1, f1| {
                        self.pre_lift.apply(t1, f1, |t2, f2| {
                            exec.run(&f2, &t2, root)
                        })
                    })
                })
        })
    }
}

// ── Public surface: seed-rooted ───────────────────────────────
//
// Treeish-rooted Stage-2 chains expose execution through the
// existing `PipelineExec::run_from_node` blanket on `TreeishSource`.
// Adding a redundant `run(exec, &root)` here would be UX clutter.

impl<Base, L> Stage2Pipeline<Base, L>
where
    Base: Stage2BaseSlice,
{
    /// Run the seed-rooted chain against an explicit `Edgy<(), Seed>`
    /// of root seeds plus an entry heap. Mirrors
    /// `SeedPipeline::run` — the `.lift()` step is hidden inside the
    /// chain, so the call shape is the same.
    pub fn run<E, CurN>(
        &self,
        exec: &E,
        root_seeds: <<Base as TreeishSource>::Domain as Domain<()>>::Graph<<Base as Stage2BaseSlice>::Seed>,
        entry_heap: <Base as TreeishSource>::H,
    ) -> L::MapR
    where
        CurN: Clone + 'static,
        <Base as TreeishSource>::Domain: Domain<()>
            + Domain<<Base as TreeishSource>::N>
            + Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>>
            + Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>,
        <Base as Stage2Base>::RunInputs<'static, CurN>: From<(
            <<Base as TreeishSource>::Domain as Domain<()>>::Graph<<Base as Stage2BaseSlice>::Seed>,
            <Base as TreeishSource>::H,
        )>,
        <Base as Stage2Base>::PreLift: Lift<
                <Base as TreeishSource>::Domain,
                <Base as TreeishSource>::N,
                <Base as TreeishSource>::H,
                <Base as TreeishSource>::R,
                N2 = <<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>,
                MapH = <Base as TreeishSource>::H,
                MapR = <Base as TreeishSource>::R,
            >,
        L: Lift<
                <Base as TreeishSource>::Domain,
                <<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>,
                <Base as TreeishSource>::H,
                <Base as TreeishSource>::R,
                N2 = <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
            >,
        L::MapH: Clone + 'static,
        L::MapR: Clone + 'static,
        E: Executor<
                <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
                L::MapR,
                <Base as TreeishSource>::Domain,
                <<Base as TreeishSource>::Domain as Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>>::Graph<
                    <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
                >,
            >,
        <<Base as TreeishSource>::Domain as Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>>::Graph<
            <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
        >: TreeOps<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>,
    {
        self.run_with_inputs::<E, CurN>(exec, (root_seeds, entry_heap).into())
    }

    /// Slice-of-seeds shorthand. Packs `seeds` into an `Edgy<(), Seed>`
    /// callback iterator and dispatches through `run`.
    pub fn run_from_slice<E, CurN>(
        &self,
        exec: &E,
        seeds: &[<Base as Stage2BaseSlice>::Seed],
        entry_heap: <Base as TreeishSource>::H,
    ) -> L::MapR
    where
        CurN: Clone + 'static,
        <Base as TreeishSource>::Domain: Domain<<Base as TreeishSource>::N>
            + Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>>
            + Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>,
        <Base as Stage2Base>::PreLift: Lift<
                <Base as TreeishSource>::Domain,
                <Base as TreeishSource>::N,
                <Base as TreeishSource>::H,
                <Base as TreeishSource>::R,
                N2 = <<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>,
                MapH = <Base as TreeishSource>::H,
                MapR = <Base as TreeishSource>::R,
            >,
        L: Lift<
                <Base as TreeishSource>::Domain,
                <<Base as Stage2Base>::Wrap as Wrap>::Of<<Base as TreeishSource>::N>,
                <Base as TreeishSource>::H,
                <Base as TreeishSource>::R,
                N2 = <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
            >,
        L::MapH: Clone + 'static,
        L::MapR: Clone + 'static,
        E: Executor<
                <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
                L::MapR,
                <Base as TreeishSource>::Domain,
                <<Base as TreeishSource>::Domain as Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>>::Graph<
                    <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
                >,
            >,
        <<Base as TreeishSource>::Domain as Domain<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>>::Graph<
            <<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>,
        >: TreeOps<<<Base as Stage2Base>::Wrap as Wrap>::Of<CurN>>,
    {
        self.run_with_inputs::<E, CurN>(
            exec,
            Base::slice_run_inputs::<CurN>(seeds, entry_heap),
        )
    }
}
