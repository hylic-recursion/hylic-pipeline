//! TreeishPipeline impls `TreeishSource` and exposes `.lift()`.
//! Execution comes from the `PipelineExec::run_from_node` blanket on
//! `TreeishSource`; the seed-rooted `.run(exec, seeds, heap)` is
//! intentionally not in scope — calling it on a TreeishPipeline is a
//! compile-time error.

use hylic::domain::Domain;
use hylic::ops::IdentityLift;
use super::TreeishPipeline;
use super::super::source::TreeishSource;
use super::super::stage2::Stage2Pipeline;

impl<D, N, H, R> TreeishSource for TreeishPipeline<D, N, H, R>
where D: Domain<N>,
      N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
      <D as Domain<N>>::Graph<N>:   Clone,
      <D as Domain<N>>::Fold<H, R>: Clone,
{
    type Domain = D;
    type N = N;
    type H = H;
    type R = R;

    fn with_treeish<T>(
        &self,
        cont: impl FnOnce(
            <D as Domain<N>>::Graph<N>,
            <D as Domain<N>>::Fold<H, R>,
        ) -> T,
    ) -> T {
        cont(self.treeish.clone(), self.fold.clone())
    }
}

impl<D, N, H, R> TreeishPipeline<D, N, H, R>
where D: Domain<N>,
      N: 'static, H: 'static, R: 'static,
{
    /// Transition to Stage 2 with an IdentityLift.
    pub fn lift(self) -> Stage2Pipeline<Self, IdentityLift> {
        Stage2Pipeline::new(self, IdentityLift)
    }
}
