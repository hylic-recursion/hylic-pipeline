//! GAT-normalisation helpers for `Stage2Pipeline`.
//!
//! Rust's trait-solver does not reduce
//! `<Shared as Domain<N>>::Grow<Seed, N>`
//! (or the equivalent Local GAT) to its concrete storage type
//! inside a generic impl body. Each free function here pins
//! `Self = Shared` or `Self = Local`, causing the GAT to
//! normalise in the function's scope. Every call is a zero-cost
//! identity at runtime.
//!
//! These helpers are used by `run.rs` (Shared) and `run_local.rs`
//! (Local) to pack and unpack `Grow`, `Graph`, and `Fold` without
//! forcing the normalisation to happen inline in each body.

use std::rc::Rc;
use std::sync::Arc;

use hylic::domain::{Domain, Shared, Local};
use hylic::domain::shared::fold::Fold as SharedFold;
use hylic::domain::local::Fold as LocalFold;
use hylic::domain::local::edgy::Edgy as LocalEdgy;
use hylic::graph::Edgy;

// ── Shared ───────────────────────────────────────────────────

#[inline]
pub(crate) fn shared_grow_as_arc<Seed: 'static, NOut: 'static>(
    g: <Shared as Domain<NOut>>::Grow<Seed, NOut>,
) -> Arc<dyn Fn(&Seed) -> NOut + Send + Sync> {
    g
}

#[inline]
pub(crate) fn shared_arc_as_grow<Seed: 'static, NOut: 'static>(
    a: Arc<dyn Fn(&Seed) -> NOut + Send + Sync>,
) -> <Shared as Domain<NOut>>::Grow<Seed, NOut> {
    a
}

#[inline]
pub(crate) fn shared_graph_as_edgy<NodeT: 'static>(
    g: <Shared as Domain<NodeT>>::Graph<NodeT>,
) -> Edgy<NodeT, NodeT> {
    g
}

#[inline]
pub(crate) fn shared_edgy_as_graph<NodeT: 'static>(
    e: Edgy<NodeT, NodeT>,
) -> <Shared as Domain<NodeT>>::Graph<NodeT> {
    e
}

#[inline]
pub(crate) fn shared_fold_as_concrete<NodeT: 'static, H: 'static, R: 'static>(
    f: <Shared as Domain<NodeT>>::Fold<H, R>,
) -> SharedFold<NodeT, H, R> {
    f
}

#[inline]
pub(crate) fn shared_concrete_as_fold<NodeT: 'static, H: 'static, R: 'static>(
    f: SharedFold<NodeT, H, R>,
) -> <Shared as Domain<NodeT>>::Fold<H, R> {
    f
}

// ── Local ────────────────────────────────────────────────────

#[inline]
pub(crate) fn local_grow_as_rc<Seed: 'static, NOut: 'static>(
    g: <Local as Domain<NOut>>::Grow<Seed, NOut>,
) -> Rc<dyn Fn(&Seed) -> NOut> {
    g
}

#[inline]
pub(crate) fn local_rc_as_grow<Seed: 'static, NOut: 'static>(
    r: Rc<dyn Fn(&Seed) -> NOut>,
) -> <Local as Domain<NOut>>::Grow<Seed, NOut> {
    r
}

#[inline]
pub(crate) fn local_graph_as_edgy<NodeT: 'static>(
    g: <Local as Domain<NodeT>>::Graph<NodeT>,
) -> LocalEdgy<NodeT, NodeT> {
    g
}

#[inline]
pub(crate) fn local_edgy_as_graph<NodeT: 'static>(
    e: LocalEdgy<NodeT, NodeT>,
) -> <Local as Domain<NodeT>>::Graph<NodeT> {
    e
}

#[inline]
pub(crate) fn local_fold_as_concrete<NodeT: 'static, H: 'static, R: 'static>(
    f: <Local as Domain<NodeT>>::Fold<H, R>,
) -> LocalFold<NodeT, H, R> {
    f
}

#[inline]
pub(crate) fn local_concrete_as_fold<NodeT: 'static, H: 'static, R: 'static>(
    f: LocalFold<NodeT, H, R>,
) -> <Local as Domain<NodeT>>::Fold<H, R> {
    f
}
