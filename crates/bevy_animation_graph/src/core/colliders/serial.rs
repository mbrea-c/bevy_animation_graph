use bevy::{
    asset::{AssetPath, Assets, LoadContext},
    math::Isometry3d,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::core::{animation_clip::EntityPath, skeleton::Skeleton};

use super::core::{ColliderConfig, ColliderShape, SkeletonColliders};

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ColliderConfigSerial {
    pub shape: ColliderShape,
    pub layers: u32,
    pub attached_to: EntityPath,
    pub offset: Isometry3d,
}

#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct SkeletonCollidersSerial {
    pub colliders: Vec<ColliderConfigSerial>,
    pub skeleton: AssetPath<'static>,
}

impl SkeletonCollidersSerial {
    pub fn from_value(value: &SkeletonColliders, skeletons: &Assets<Skeleton>) -> Option<Self> {
        let skeleton = skeletons.get(&value.skeleton)?;
        let mut colliders = Vec::with_capacity(value.colliders.len());
        for config in &value.colliders {
            let config_serial = ColliderConfigSerial {
                shape: config.shape.clone(),
                layers: config.layers,
                attached_to: skeleton.id_to_path(config.attached_to)?,
                offset: config.offset,
            };

            colliders.push(config_serial);
        }

        Some(Self {
            colliders,
            skeleton: value.skeleton.path()?.to_owned(),
        })
    }

    pub async fn to_value(&self, load_context: &mut LoadContext<'_>) -> Option<SkeletonColliders> {
        let skeleton_handle = load_context.load(&self.skeleton);
        let skeleton: Skeleton = load_context
            .loader()
            .immediate()
            .load(&self.skeleton)
            .await
            .ok()?
            .take();

        let mut colliders = Vec::with_capacity(self.colliders.len());
        for config_serial in &self.colliders {
            let config = ColliderConfig {
                shape: config_serial.shape.clone(),
                layers: config_serial.layers,
                attached_to: skeleton.path_to_id(config_serial.attached_to.clone())?,
                offset: config_serial.offset,
            };

            colliders.push(config);
        }

        Some(SkeletonColliders {
            colliders,
            skeleton: skeleton_handle,
        })
    }
}
