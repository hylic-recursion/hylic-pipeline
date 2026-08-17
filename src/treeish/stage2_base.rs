//! `Stage2Base` impl for `TreeishPipeline`.
//!
//! Single generic-over-D impl. The pre-lift is `IdentityLift` (no
//! transformation); the root is the user's post-chain `&CurN`,
//! threaded straight through `RunInputs`.

use hylic::domain::Domain;
use hylic::ops::IdentityLift;

use super::TreeishPipeline;
use crate::stage2::{Stage2Base, Wrap};

impl<D, N, H, R> Stage2Base for TreeishPipeline<D, N, H, R>
where
    D: Domain<N>,
    N: Clone + 'static,
    H: Clone + 'static,
    R: Clone + 'static,
    <D as Domain<N>>::Graph<N>: Clone,
    <D as Domain<N>>::Fold<H, R>: Clone,
{
    type Wrap = crate::stage2::Identity;
    type UserN = N;
    type RunInputs<'i, CurN: Clone + 'static> = &'i CurN;
    type PreLift = IdentityLift;

    fn provide_run_essentials<CurN: Clone + 'static, T>(
        &self,
        inputs: Self::RunInputs<'_, CurN>,
        cont: impl FnOnce(Self::PreLift, &<Self::Wrap as Wrap>::Of<CurN>) -> T,
    ) -> T {
        // For `Identity`-wrap bases, `<Wrap as Wrap>::Of<CurN> = CurN`,
        // so the root reference is exactly the borrowed `&CurN` from
        // inputs.
        cont(IdentityLift, inputs)
    }
}
