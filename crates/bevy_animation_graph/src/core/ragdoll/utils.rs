use crate::core::{
    pose::Pose,
    ragdoll::{bone_mapping::RagdollBoneMap, definition::Ragdoll},
    skeleton::Skeleton,
};

/// Computes the offsets in a ragdoll bone map based on the initial positioning of the ragdoll and
/// given pose.
pub fn create_mapping(
    ragdoll: &Ragdoll,
    pose: &Pose,
    skeleton: &Skeleton,
    base_mapping: &RagdollBoneMap,
) -> RagdollBoneMap {
    let mapping = base_mapping.clone();

    mapping
}
