use bevy::asset::{AssetLoader, LoadContext, io::Reader};

use crate::core::{errors::AssetLoaderError, ragdoll::definition::Ragdoll};

#[derive(Default)]
pub struct RagdollLoader;

impl AssetLoader for RagdollLoader {
    type Asset = Ragdoll;
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
        let ragdoll: Ragdoll = ron::de::from_bytes(&bytes)?;

        Ok(ragdoll)
    }

    fn extensions(&self) -> &[&str] {
        &["rag.ron"]
    }
}
