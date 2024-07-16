use bevy::{
    animation::AnimationTargetId,
    reflect::{std_traits::ReflectDefault, Reflect},
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

use super::animation_clip::EntityPath;

#[derive(
    Reflect, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, Debug,
)]
#[reflect(Default)]
pub struct BoneId {
    id: Uuid,
}

impl Hash for BoneId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (hi, lo) = self.id.as_u64_pair();
        state.write_u64(hi ^ lo);
    }
}

impl BoneId {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn animation_target_id(&self) -> AnimationTargetId {
        AnimationTargetId(self.id)
    }
}

impl From<EntityPath> for BoneId {
    fn from(value: EntityPath) -> Self {
        value.id()
    }
}

impl From<AnimationTargetId> for BoneId {
    fn from(value: AnimationTargetId) -> Self {
        BoneId { id: value.0 }
    }
}
