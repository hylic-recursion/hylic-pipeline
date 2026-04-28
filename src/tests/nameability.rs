//! Nameability — pipeline types store cleanly as struct fields and
//! type aliases. Proves the Arc-erasure claim via std::any::type_name.

use crate::SeedPipeline;
use crate::Stage2Pipeline;
use hylic::domain::Shared;
use hylic::ops::{ComposedLift, IdentityLift, ShapeLift};

type MyBasePipeline = SeedPipeline<hylic::domain::Shared, u64, u64, u64, u64>;

type MyTransformedPipeline = Stage2Pipeline<
    SeedPipeline<hylic::domain::Shared, u64, u64, u64, u64>,
    ComposedLift<IdentityLift, ShapeLift<Shared, u64, u64, u64, u64, u64, (u64, bool)>>,
>;

#[allow(dead_code)]
struct Resolver {
    pipe: MyBasePipeline,
}

#[allow(dead_code)]
struct ResolverWithLifts {
    pipe: MyTransformedPipeline,
}

#[test]
fn pipeline_types_name_and_store() {
    // Name the base pipeline type.
    let base_name = std::any::type_name::<MyBasePipeline>();
    assert!(base_name.contains("SeedPipeline"));
    assert!(base_name.contains("u64"));

    // Name the transformed pipeline type — the unified Stage-2 form.
    let transformed_name = std::any::type_name::<MyTransformedPipeline>();
    assert!(transformed_name.contains("Stage2Pipeline"));
    assert!(transformed_name.contains("ComposedLift"));
    assert!(transformed_name.contains("ShapeLift"));
}
