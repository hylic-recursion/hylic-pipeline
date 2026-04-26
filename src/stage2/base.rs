//! `Stage2Base` trait — declares which `Wrap` a Stage-1 base uses
//! for its Stage-2 chain.
//!
//! Implemented for both Stage-1 base types:
//!
//! - `TreeishPipeline<D, N, H, R>` uses `Wrap = Identity`. The chain
//!   operates on plain N; chain-tip MapR types never mention
//!   `SeedNode<N>`.
//!
//! - `SeedPipeline<D, N, Seed, H, R>` uses `Wrap = SeedWrap`. The
//!   chain operates on `SeedNode<N>` internally; sugars dispatch
//!   user closures over `&N` via `SeedNode::as_node()`.
//!
//! `Stage2Base` extends `TreeishSource` (the "yield (treeish, fold)"
//! interface), so a Stage-2 pipeline composed over any `Stage2Base`
//! can be executed via `run_from_node` on the wrapped pair.

use hylic::domain::Domain;
use hylic::ops::ShapeCapable;
use crate::source::TreeishSource;
use crate::seed::SeedPipeline;
use crate::treeish::TreeishPipeline;
use super::wrap::{Identity, SeedWrap, Wrap};

// ANCHOR: stage2_base_trait
/// A Stage-1 pipeline that can transition to Stage 2 via `.lift()`.
/// Declares the `Wrap` its Stage-2 chain uses for the input N.
pub trait Stage2Base: TreeishSource + Sized {
    /// The wrap for the Stage-2 chain's input N.
    type Wrap: Wrap;

    /// The user-facing node type that user lambdas type at. For
    /// `TreeishPipeline`: same as `Self::N`. For `SeedPipeline`:
    /// the base node type N (Self::N is `SeedNode<N>` in the chain;
    /// UserN unwraps the seed wrapping for user-facing closures).
    type UserN: Clone + 'static;
}
// ANCHOR_END: stage2_base_trait

impl<D, N, H, R> Stage2Base for TreeishPipeline<D, N, H, R>
where D: Domain<N>,
      N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
      <D as Domain<N>>::Graph<N>:   Clone,
      <D as Domain<N>>::Fold<H, R>: Clone,
{
    type Wrap  = Identity;
    type UserN = N;
}

impl<D, N, Seed, H, R> Stage2Base for SeedPipeline<D, N, Seed, H, R>
where D: ShapeCapable<N>,
      N: Clone + 'static, Seed: Clone + 'static,
      H: Clone + 'static, R: Clone + 'static,
      <D as Domain<N>>::Grow<Seed, N>: Clone,
      <D as Domain<N>>::Graph<Seed>:   Clone,
      <D as Domain<N>>::Fold<H, R>:    Clone,
{
    type Wrap  = SeedWrap;
    type UserN = N;
}
