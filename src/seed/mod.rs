//! SeedPipeline — Stage 1 of the Phase-3 typestate. Holds the three
//! base slots (grow, seeds_from_node, fold) in the domain D's
//! native storage. Sole primitive: reshape.

use hylic::domain::Domain;

pub mod reshape;
pub mod source;
pub mod stage2_base_shared;
pub mod stage2_base_local;
pub mod run;

// ANCHOR: seed_pipeline_struct
/// Stage-1 typestate pipeline with three base slots: `grow`,
/// `seeds_from_node`, and `fold`. Used when the tree is discovered
/// lazily from `Seed` references.
#[must_use]
pub struct SeedPipeline<D, N, Seed, H, R>
where D: Domain<N>,
      N: 'static, Seed: 'static, H: 'static, R: 'static,
{
    pub(crate) grow:            <D as Domain<N>>::Grow<Seed, N>,
    pub(crate) seeds_from_node: <D as Domain<N>>::Graph<Seed>,
    pub(crate) fold:            <D as Domain<N>>::Fold<H, R>,
}
// ANCHOR_END: seed_pipeline_struct

impl<D, N, Seed, H, R> Clone for SeedPipeline<D, N, Seed, H, R>
where D: Domain<N>,
      N: 'static, Seed: 'static, H: 'static, R: 'static,
      <D as Domain<N>>::Grow<Seed, N>: Clone,
      <D as Domain<N>>::Graph<Seed>:   Clone,
      <D as Domain<N>>::Fold<H, R>:    Clone,
{
    fn clone(&self) -> Self {
        SeedPipeline {
            grow:            self.grow.clone(),
            seeds_from_node: self.seeds_from_node.clone(),
            fold:            self.fold.clone(),
        }
    }
}

impl<D, N, Seed, H, R> SeedPipeline<D, N, Seed, H, R>
where D: Domain<N>,
      N: 'static, Seed: 'static, H: 'static, R: 'static,
{
    /// Construct from already-domain-typed slots (grow, seeds,
    /// fold). For the common path where the caller has plain
    /// closures instead, use the per-domain `new` inherent method
    /// (e.g. on `SeedPipeline<Shared, ...>` or
    /// `SeedPipeline<Local, ...>`).
    pub fn from_slots(
        grow:            <D as Domain<N>>::Grow<Seed, N>,
        seeds_from_node: <D as Domain<N>>::Graph<Seed>,
        fold:            <D as Domain<N>>::Fold<H, R>,
    ) -> Self {
        SeedPipeline { grow, seeds_from_node, fold }
    }
}

// ── Shared convenience constructor ─────────────────────

impl<N, Seed, H, R> SeedPipeline<hylic::domain::Shared, N, Seed, H, R>
where N: 'static, Seed: 'static, H: 'static, R: 'static,
{
    /// Shared-specific constructor that takes a plain Fn closure for
    /// grow and an `Edgy<N, Seed>` for seeds_from_node. Mirrors the
    /// pre-5/5 API for Shared users.
    pub fn new(
        grow: impl Fn(&Seed) -> N + Send + Sync + 'static,
        seeds_from_node: hylic::graph::Edgy<N, Seed>,
        fold: &hylic::domain::shared::fold::Fold<N, H, R>,
    ) -> Self {
        SeedPipeline {
            grow: std::sync::Arc::new(grow),
            seeds_from_node,
            fold: fold.clone(),
        }
    }
}

// ── Local convenience constructor ──────────────────────

impl<N, Seed, H, R> SeedPipeline<hylic::domain::Local, N, Seed, H, R>
where N: 'static, Seed: 'static, H: 'static, R: 'static,
{
    /// Local-specific constructor. Rc-backed `grow`; `seeds_from_node`
    /// is a Local-edgy over `(N, Seed)`; `fold` is the Local Fold.
    /// Closures need not be `Send + Sync`.
    pub fn new_local(
        grow: impl Fn(&Seed) -> N + 'static,
        seeds_from_node: hylic::domain::local::edgy::Edgy<N, Seed>,
        fold: &hylic::domain::local::Fold<N, H, R>,
    ) -> Self {
        SeedPipeline {
            grow: std::rc::Rc::new(grow),
            seeds_from_node,
            fold: fold.clone(),
        }
    }
}
