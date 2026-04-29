//! GAT-normalisation helpers for `Stage2Pipeline`.
//!
//! Rust's trait-solver does not reduce
//! `<Shared as Domain<N>>::Grow<Seed, N>`
//! (or the equivalent Local GAT) to its concrete storage type
//! (`Arc<dyn Fn…>` / `Rc<dyn Fn…>`) inside a generic-over-D impl
//! body. Each free function here pins `Self = Shared` or
//! `Self = Local`, causing the GAT to normalise in the function's
//! scope. Every call is a zero-cost identity at runtime.
//!
//! These helpers are used by the per-domain `Stage2Base` impls in
//! `seed/stage2_base_*.rs` to convert the SeedPipeline's stored
//! grow-GAT into the `Arc/Rc` shape that `SeedLift::from_*_grow`
//! expects.
//!
//! The graph/fold conversion helpers that previously lived here were
//! retired when the unified Stage-2 run body switched from manual
//! `D::fuse_grow_with_seeds` re-fusion to going through
//! `TreeishSource::with_treeish` (which already speaks the GAT shape
//! the executor needs).

use std::rc::Rc;
use std::sync::Arc;

use hylic::domain::{Domain, Shared, Local};

#[inline]
pub(crate) fn shared_grow_as_arc<Seed: 'static, NOut: 'static>(
    g: <Shared as Domain<NOut>>::Grow<Seed, NOut>,
) -> Arc<dyn Fn(&Seed) -> NOut + Send + Sync> {
    g
}

#[inline]
pub(crate) fn local_grow_as_rc<Seed: 'static, NOut: 'static>(
    g: <Local as Domain<NOut>>::Grow<Seed, NOut>,
) -> Rc<dyn Fn(&Seed) -> NOut> {
    g
}
