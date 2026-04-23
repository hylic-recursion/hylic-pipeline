//! OwnedPipeline — one-shot, seedless, by-value pipeline over the
//! Owned domain.
//!
//! Not Clone; run_from_node_once consumes self. Not ShapeCapable —
//! doesn't compose shape-lifts. Impls `PipelineSourceOnce` (by-value
//! analogue of TreeishSource).

use hylic::domain::{Domain, Owned};
use hylic::domain::owned::Fold;
use hylic::domain::owned::edgy::Edgy;
use super::source::PipelineSourceOnce;

// ANCHOR: owned_pipeline_struct
/// One-shot pipeline over the `Owned` domain. Not `Clone`; runs
/// via [`crate::source::PipelineExecOnce::run_from_node_once`],
/// which consumes `self`.
#[must_use = "an OwnedPipeline is consumed by `run_from_node_once(...)`; constructing it without running has no effect"]
pub struct OwnedPipeline<N, H, R>
where N: 'static, H: 'static, R: 'static,
{
    pub(crate) treeish: Edgy<N, N>,
    pub(crate) fold:    Fold<N, H, R>,
}
// ANCHOR_END: owned_pipeline_struct

impl<N, H, R> OwnedPipeline<N, H, R>
where N: 'static, H: 'static, R: 'static,
{
    /// Construct an owned pipeline from its two slots.
    pub fn new(treeish: Edgy<N, N>, fold: Fold<N, H, R>) -> Self {
        OwnedPipeline { treeish, fold }
    }
}

impl<N, H, R> PipelineSourceOnce for OwnedPipeline<N, H, R>
where N: 'static, H: 'static, R: 'static,
{
    type Domain = Owned;
    type N    = N;
    type H    = H;
    type R    = R;

    fn with_constructed_once<T>(
        self,
        cont: impl FnOnce(
            <Owned as Domain<N>>::Graph<N>,
            <Owned as Domain<N>>::Fold<H, R>,
        ) -> T,
    ) -> T {
        cont(self.treeish, self.fold)
    }
}
