//! `Wrap` trait — type-level dispatch for the chain's input N.
//!
//! A Stage-2 chain's input N is `<Wrap as Wrap>::Of<UN>`, where `UN`
//! is the user-facing N (the type user lambdas type at):
//!
//! - [`Identity`] — `Of<UN> = UN`. Treeish-rooted chains.
//! - [`SeedWrap`] — `Of<UN> = SeedNode<UN>`. Seed-rooted chains.
//!
//! [`Stage2Base`](super::Stage2Base) selects which `Wrap` applies for
//! each Stage-1 base. The actual lift-construction lives on the
//! per-domain subtraits [`WrapShared`](super::WrapShared) /
//! [`WrapLocal`](super::WrapLocal); this trait carries only the type
//! family.
//!
//! Sugars chain through both: the `Stage2SugarsShared`/`Local` body
//! references `<<Self::Base as Stage2Base>::Wrap as Wrap>::Of<UN>` in
//! its return type and calls
//! `<<Base::Wrap as WrapShared>::build_*` to construct the actual
//! `ShapeLift`. The two-hop projection appears symmetrically in both
//! positions; a single chain reduces and the bound check passes.

use hylic::ops::SeedNode;

pub mod local;
pub mod shared;

// ANCHOR: wrap_trait
/// Type-level dispatch for the chain's input N. Each
/// [`Stage2Base`](super::Stage2Base) declares which `Wrap` it uses;
/// `WrapShared` / `WrapLocal` impls carry the per-domain lift
/// construction.
pub trait Wrap {
    /// The wrapped node type for a given user-facing N.
    type Of<UN: Clone + 'static>: Clone + 'static;
}
// ANCHOR_END: wrap_trait

/// Identity wrap: `Of<UN> = UN`. Used by `TreeishPipeline`-rooted
/// Stage-2 chains. Sugars apply user closures directly.
#[derive(Clone, Copy, Debug, Default)]
pub struct Identity;

impl Wrap for Identity {
    type Of<UN: Clone + 'static> = UN;
}

/// Seed wrap: `Of<UN> = SeedNode<UN>`. Used by `SeedPipeline`-rooted
/// Stage-2 chains. `WrapShared`/`WrapLocal` impls peel
/// `SeedNode::Node(_)` for N-aware sugars; `EntryRoot` passes through
/// to the chain's `orig` continuation.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeedWrap;

impl Wrap for SeedWrap {
    type Of<UN: Clone + 'static> = SeedNode<UN>;
}
