use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation_clip::EntityPath,
    errors::AssetLoaderError,
    ragdoll::{
        bone_mapping::{BodyMapping, BoneMapping, RagdollBoneMap},
        definition::BodyId,
    },
    symmetry::serial::SymmetryConfigSerial,
};

#[derive(Default)]
pub struct RagdollBoneMapLoader;

impl AssetLoader for RagdollBoneMapLoader {
    type Asset = RagdollBoneMap;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let RagdollBoneMapSerial {
            bones_from_bodies,
            bodies_from_bones,
            skeleton,
            ragdoll,
            skeleton_symmetry,
        } = ron::de::from_bytes(&bytes)?;

        Ok(RagdollBoneMap {
            bones_from_bodies,
            bodies_from_bones,
            skeleton: load_context.load(skeleton),
            ragdoll: load_context.load(ragdoll),
            skeleton_symmetry: skeleton_symmetry.to_value()?,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bm.ron"]
    }
}

#[derive(Debug, Clone, Reflect, Default, Serialize, Deserialize)]
pub struct RagdollBoneMapSerial {
    pub bones_from_bodies: HashMap<EntityPath, BoneMapping>,
    pub bodies_from_bones: HashMap<BodyId, BodyMapping>,
    pub skeleton: AssetPath<'static>,
    pub ragdoll: AssetPath<'static>,
    #[serde(default)]
    pub skeleton_symmetry: SymmetryConfigSerial,
}

impl RagdollBoneMapSerial {
    pub fn from_value(ragdoll_bone_map: &RagdollBoneMap) -> Option<Self> {
        let RagdollBoneMap {
            bones_from_bodies,
            bodies_from_bones,
            skeleton,
            ragdoll,
            skeleton_symmetry,
        } = ragdoll_bone_map;

        Some(Self {
            bones_from_bodies: bones_from_bodies.clone(),
            bodies_from_bones: bodies_from_bones.clone(),
            skeleton: skeleton.path()?.to_owned(),
            ragdoll: ragdoll.path()?.to_owned(),
            skeleton_symmetry: SymmetryConfigSerial::from_value(skeleton_symmetry),
        })
    }
}
