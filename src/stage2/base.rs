//! `Stage2Base` — the bridge from a Stage-1 base to its Stage-2 chain.
//!
//! Implemented for both Stage-1 base types (`TreeishPipeline`,
//! `SeedPipeline`). Carries:
//!
//! - The type-level [`Wrap`] selecting how the chain wraps the user's N
//!   (`Identity` for treeish-rooted, `SeedWrap` for seed-rooted).
//! - The run-time machinery — `RunInputs`, `PreLift`,
//!   `provide_run_essentials` — that makes one generic
//!   `Stage2Pipeline::run` body cover every base.
//!
//! The actual `Stage2Base` impls live in the seed/treeish subtrees:
//!
//! - `treeish::stage2_base` — single generic-over-D impl
//! - `seed::stage2_base_shared` / `stage2_base_local` — one impl per
//!   domain (the GAT crossing for `Domain::Grow<Seed, N>` forces this;
//!   bodies are ~15 lines each).

use crate::source::TreeishSource;
use super::wrap::Wrap;

// ANCHOR: stage2_base_trait
/// A Stage-1 pipeline that can drive a Stage-2 chain. Carries the
/// `Wrap` selection plus the run-time machinery (pre-lift, root
/// reference, run-input shape).
///
/// Inherits `TreeishSource` so the (treeish<N>, fold<N,H,R>) pair is
/// yielded through one canonical path; `with_treeish` is the single
/// place per-base storage shapes are read.
///
/// `PreLift` is intentionally unbounded at the trait level. The
/// `Stage2Pipeline::run` impl adds the `Lift<…, N2 = <Wrap>::Of<N>>`
/// bound at use time; that keeps the supertrait surface free of the
/// `Domain<<Wrap>::Of<N>>` obligation that would otherwise propagate
/// through every site naming `Stage2Base`.
pub trait Stage2Base: TreeishSource + Sized {
    /// Type-level dispatcher for the chain's input N.
    /// `Identity` → `Of<UN> = UN` (treeish-rooted).
    /// `SeedWrap` → `Of<UN> = SeedNode<UN>` (seed-rooted).
    type Wrap: Wrap;

    /// The user-facing N (the type user lambdas type at). Equal to
    /// `Self::N` for every shipped base; kept distinct for
    /// documentation symmetry with the sugar surface, which threads
    /// `UN` as a method-level parameter.
    type UserN: Clone + 'static;

    /// What `.run(...)` accepts as its second argument. Parameterised
    /// by `CurN`, the user-facing N at the chain tip (i.e. after any
    /// `map_n_bi` lifts; `CurN = Self::N` if the chain doesn't change
    /// the user N).
    ///
    /// `Identity`-Wrap bases: `&'i CurN` (a borrowed post-chain root).
    /// `SeedWrap` bases: an owned `(seeds, entry_heap)` pair (the
    /// `CurN` parameter is unused at the value level — `EntryRoot` is
    /// constructible at any inner type).
    type RunInputs<'i, CurN: Clone + 'static>;

    /// The lift composed at the head of the run-time chain.
    /// `IdentityLift` for treeish-rooted, `SeedLift` for seed-rooted.
    /// Pre-lift transforms `(treeish<N>, fold<N,H,R>)` into
    /// `(treeish<Wrap::Of<N>>, fold<Wrap::Of<N>, H, R>)` without
    /// touching H or R.
    ///
    /// Unbounded at the trait level — see the trait-level note.
    /// The `Stage2Pipeline::run` impl adds
    /// `Self::PreLift: Lift<…, N2 = <Wrap>::Of<N>, MapH = H, MapR = R>`
    /// at use time.
    type PreLift;

    /// Build the pre-lift from inputs (consuming the parts of inputs
    /// the lift captures), then yield it together with the executor's
    /// post-chain root reference to the continuation.
    ///
    /// The continuation receives the pre-lift by value (consumed when
    /// applied to the (treeish, fold) pair) and the root by reference,
    /// at the post-chain type `<Self::Wrap as Wrap>::Of<CurN>`. The
    /// reference is valid for the entire duration of `cont`.
    ///
    /// `Identity` case: pre-lift is `IdentityLift`; the root is the
    /// `&CurN` extracted from `inputs`.
    /// `SeedWrap` case: pre-lift is `SeedLift::from_*_grow(...)`,
    /// consuming `inputs.0` (entry seeds) and `inputs.1` (entry heap);
    /// the root is `&SeedNode::entry_root::<CurN>()`, constructed
    /// locally in this frame and alive for `cont`'s lifetime.
    fn provide_run_essentials<CurN: Clone + 'static, T>(
        &self,
        inputs: Self::RunInputs<'_, CurN>,
        cont: impl FnOnce(Self::PreLift,
                          &<Self::Wrap as Wrap>::Of<CurN>) -> T,
    ) -> T;
}
// ANCHOR_END: stage2_base_trait

/// Optional companion to [`Stage2Base`] for bases whose `RunInputs`
/// can be constructed from a `&[Seed]` slice plus an entry heap.
/// Implemented only for `SeedPipeline<D, …>`. Powers
/// `Stage2Pipeline::run_from_slice`.
pub trait Stage2BaseSlice: Stage2Base {
    /// The seed type the base grows from.
    type Seed: Clone + 'static;

    /// Pack a slice of seeds and an entry heap into the base's
    /// `RunInputs<'static, CurN>`. The slice is cloned into an owned
    /// callback iterator; the result borrows nothing from the slice.
    /// `CurN` is the post-chain user N — irrelevant at the value
    /// level for SeedWrap bases (EntryRoot is N-agnostic), but
    /// threaded through so the GAT projection lines up.
    fn slice_run_inputs<CurN: Clone + 'static>(
        seeds: &[Self::Seed],
        heap: <Self as TreeishSource>::H,
    ) -> Self::RunInputs<'static, CurN>;
}
