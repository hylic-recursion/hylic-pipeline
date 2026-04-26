//! `Wrap` trait — type-level dispatch for chain N wrapping.
//!
//! A Stage-2 chain's input N is `Wrap::Of<UN>` where `UN` is the
//! "user-facing N" (what user lambdas type at). Two implementations:
//!
//! - [`Identity`]: `Of<UN> = UN`. Used by `TreeishPipeline`-rooted
//!   chains. The chain operates on plain N. `project` is total
//!   (always returns `Some`); `wrap` is identity; `map` is direct
//!   function application.
//!
//! - [`SeedWrap`]: `Of<UN> = SeedNode<UN>`. Used by `SeedPipeline`-rooted
//!   chains. The chain operates on `SeedNode<UN>` (`EntryRoot |
//!   Node(UN)`). `project` returns `Some` only on `Node(_)` and
//!   `None` on `EntryRoot`; `wrap` constructs `Node(_)`; `map`
//!   preserves `EntryRoot` and applies `f` inside `Node(_)`.
//!
//! Stage-2 sugars dispatch user closures via these methods. Under
//! `Identity`, the dispatch is a no-op (the optimiser elides the
//! `match` arm corresponding to `EntryRoot`); under `SeedWrap`, the
//! `EntryRoot` arm passes through to the chain's `orig` continuation.

use hylic::ops::{SeedNode, seed_node_internal::{self, SeedNodeInner}};

// ANCHOR: wrap_trait
/// Type-level dispatch for the chain's input N wrapping. Each
/// `Stage2Base` declares which `Wrap` it uses; sugars are written
/// once against this trait.
pub trait Wrap {
    /// The wrapped node type for a given user-facing N.
    type Of<UN: Clone + 'static>: Clone + 'static;

    /// Project a wrapped node back to its user-facing form.
    /// Returns `None` for synthetic positions (e.g. `EntryRoot`
    /// under `SeedWrap`); user closures over `&UN` then pass through
    /// to the chain's continuation.
    fn project<UN: Clone + 'static>(w: &Self::Of<UN>) -> Option<&UN>;

    /// Wrap a user-facing N as the chain's input N. Inverse of
    /// `project` on resolved nodes.
    fn wrap<UN: Clone + 'static>(un: UN) -> Self::Of<UN>;

    /// Functorial map over the wrap: apply `f` inside the wrap.
    /// For `Identity`: `map(w, f) = f(w)`. For `SeedWrap`: preserve
    /// `EntryRoot`, apply `f` inside `Node(_)`. Used by
    /// `map_n_bi`-style sugars to thread N changes through the wrap.
    fn map<UN1: Clone + 'static, UN2: Clone + 'static>(
        w: Self::Of<UN1>,
        f: impl FnOnce(UN1) -> UN2,
    ) -> Self::Of<UN2>;
}
// ANCHOR_END: wrap_trait

/// Identity wrap: `Of<UN> = UN`. Used by `TreeishPipeline`-rooted
/// Stage-2 chains. All projections are total.
#[derive(Clone, Copy, Debug, Default)]
pub struct Identity;

impl Wrap for Identity {
    type Of<UN: Clone + 'static> = UN;

    #[inline]
    fn project<UN: Clone + 'static>(w: &UN) -> Option<&UN> { Some(w) }

    #[inline]
    fn wrap<UN: Clone + 'static>(un: UN) -> UN { un }

    #[inline]
    fn map<UN1: Clone + 'static, UN2: Clone + 'static>(
        w: UN1,
        f: impl FnOnce(UN1) -> UN2,
    ) -> UN2 {
        f(w)
    }
}

/// Seed wrap: `Of<UN> = SeedNode<UN>`. Used by `SeedPipeline`-rooted
/// Stage-2 chains. `project` is partial — `None` on `EntryRoot`,
/// `Some(&n)` on `Node(n)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeedWrap;

impl Wrap for SeedWrap {
    type Of<UN: Clone + 'static> = SeedNode<UN>;

    #[inline]
    fn project<UN: Clone + 'static>(w: &SeedNode<UN>) -> Option<&UN> {
        w.as_node()
    }

    #[inline]
    fn wrap<UN: Clone + 'static>(un: UN) -> SeedNode<UN> {
        seed_node_internal::node(un)
    }

    #[inline]
    fn map<UN1: Clone + 'static, UN2: Clone + 'static>(
        w: SeedNode<UN1>,
        f: impl FnOnce(UN1) -> UN2,
    ) -> SeedNode<UN2> {
        match w.into_inner() {
            SeedNodeInner::Node(n)   => seed_node_internal::node(f(n)),
            SeedNodeInner::EntryRoot => seed_node_internal::entry_root(),
        }
    }
}
