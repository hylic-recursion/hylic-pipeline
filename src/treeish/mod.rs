//! TreeishPipeline — the honest-base pipeline for users who have
//! a `Treeish<N>` directly (no `grow: Seed → N` step). Two base
//! slots: `treeish` and `fold`. `Self::Seed = ()` — no Seed
//! dispatch at the executor boundary; use `run_from_node`.

use hylic::domain::Domain;

pub mod reshape;
pub mod source_impl;

// ANCHOR: treeish_pipeline_struct
/// Stage-1 typestate pipeline with two base slots: `treeish`
/// (graph) and `fold`. Used when children are directly enumerable
/// from nodes of the same type (`N → N*`).
#[must_use = "a TreeishPipeline carries the transformation plan; call `.run_from_node(...)` to execute it"]
pub struct TreeishPipeline<D, N, H, R>
where D: Domain<N>,
      N: 'static, H: 'static, R: 'static,
{
    pub(crate) treeish: <D as Domain<N>>::Graph<N>,
    pub(crate) fold:    <D as Domain<N>>::Fold<H, R>,
}
// ANCHOR_END: treeish_pipeline_struct

impl<D, N, H, R> Clone for TreeishPipeline<D, N, H, R>
where D: Domain<N>,
      N: 'static, H: 'static, R: 'static,
      <D as Domain<N>>::Graph<N>:   Clone,
      <D as Domain<N>>::Fold<H, R>: Clone,
{
    fn clone(&self) -> Self {
        TreeishPipeline {
            treeish: self.treeish.clone(),
            fold:    self.fold.clone(),
        }
    }
}

impl<D, N, H, R> TreeishPipeline<D, N, H, R>
where D: Domain<N>,
      N: 'static, H: 'static, R: 'static,
{
    /// Construct from already-domain-typed slots. For the common
    /// path where the caller has plain closures, use the per-domain
    /// `new` inherent method below.
    pub fn from_slots(
        treeish: <D as Domain<N>>::Graph<N>,
        fold:    <D as Domain<N>>::Fold<H, R>,
    ) -> Self {
        TreeishPipeline { treeish, fold }
    }
}

// ── Shared convenience constructor ─────────────────────

impl<N, H, R> TreeishPipeline<hylic::domain::Shared, N, H, R>
where N: 'static, H: 'static, R: 'static,
{
    /// Construct a Shared-domain pipeline from an Arc-backed
    /// `Treeish<N>` and a borrowed `Fold<N, H, R>` (cloned in).
    pub fn new(
        treeish: hylic::graph::Treeish<N>,
        fold:    &hylic::domain::shared::fold::Fold<N, H, R>,
    ) -> Self {
        TreeishPipeline { treeish, fold: fold.clone() }
    }
}

// ── Local convenience constructor ──────────────────────

impl<N, H, R> TreeishPipeline<hylic::domain::Local, N, H, R>
where N: 'static, H: 'static, R: 'static,
{
    /// Construct a Local-domain pipeline from an Rc-backed treeish
    /// and fold. Non-`Send` captures are accepted; execution is
    /// single-threaded (Fused).
    ///
    /// The name differs from the Shared-domain `new` because Rust's
    /// inherent-method resolution does not disambiguate two `new`
    /// fns on the same struct under different generic parameters
    /// when the call site writes `TreeishPipeline::new(...)`
    /// without a `::<Domain, ...>` turbofish.
    pub fn new_local(
        treeish: hylic::domain::local::edgy::Edgy<N, N>,
        fold:    hylic::domain::local::Fold<N, H, R>,
    ) -> Self {
        TreeishPipeline { treeish, fold }
    }
}
