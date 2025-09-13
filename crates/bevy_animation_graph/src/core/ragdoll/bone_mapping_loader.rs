use bevy::asset::{AssetLoader, LoadContext, io::Reader};

use crate::core::{errors::AssetLoaderError, ragdoll::bone_mapping::RagdollBoneMap};

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
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let map: RagdollBoneMap = ron::de::from_bytes(&bytes)?;

        Ok(map)
    }

    fn extensions(&self) -> &[&str] {
        &["bm.ron"]
    }
}
