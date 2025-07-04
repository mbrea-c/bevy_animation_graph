use bevy::{
    asset::{AssetPath, Assets, LoadContext},
    math::Isometry3d,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{animation_clip::EntityPath, colliders::core::SkeletonColliderId, skeleton::Skeleton},
    prelude::serial::SymmetryConfigSerial,
};

use super::core::{ColliderConfig, ColliderOffsetMode, ColliderShape, SkeletonColliders};

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ColliderConfigSerial {
    pub id: SkeletonColliderId,
    pub shape: ColliderShape,
    pub override_layers: bool,
    pub layer_membership: u32,
    pub layer_filter: u32,
    pub attached_to: EntityPath,
    pub offset: Isometry3d,
    #[serde(default)]
    pub offset_mode: ColliderOffsetMode,
}

#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct SkeletonCollidersSerial {
    #[serde(default)]
    pub colliders: Vec<ColliderConfigSerial>,
    pub skeleton: AssetPath<'static>,
    #[serde(default)]
    pub symmetry: SymmetryConfigSerial,
    #[serde(default)]
    pub symmetry_enabled: bool,
}

impl SkeletonCollidersSerial {
    pub fn from_value(value: &SkeletonColliders, skeletons: &Assets<Skeleton>) -> Option<Self> {
        let skeleton = skeletons.get(&value.skeleton)?;
        let mut colliders = Vec::with_capacity(value.collider_count());
        for config in value.iter_colliders() {
            let config_serial = ColliderConfigSerial {
                shape: config.shape.clone(),
                override_layers: config.override_layers,
                layer_membership: config.layer_membership,
                layer_filter: config.layer_filter,
                attached_to: skeleton.id_to_path(config.attached_to)?,
                offset: config.offset,
                id: config.id,
                offset_mode: config.offset_mode,
            };

            colliders.push(config_serial);
        }

        Some(Self {
            colliders,
            skeleton: value.skeleton.path()?.to_owned(),
            symmetry: SymmetryConfigSerial::from_value(&value.symmetry),
            symmetry_enabled: value.symmetry_enabled,
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

        let mut colliders = SkeletonColliders::default();

        colliders.skeleton = skeleton_handle;
        colliders.symmetry = self.symmetry.to_value().ok()?;
        colliders.symmetry_enabled = self.symmetry_enabled;

        for config_serial in &self.colliders {
            let config = ColliderConfig {
                shape: config_serial.shape.clone(),
                override_layers: config_serial.override_layers,
                layer_membership: config_serial.layer_membership,
                layer_filter: config_serial.layer_filter,
                attached_to: skeleton.path_to_id(config_serial.attached_to.clone())?,
                offset: config_serial.offset,
                id: config_serial.id,
                offset_mode: config_serial.offset_mode,
            };

            colliders.add_collider(config);
        }

        Some(colliders)
    }
}
