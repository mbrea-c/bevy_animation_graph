use bevy::asset::{AssetLoader, LoadContext, io::Reader};

use crate::core::errors::AssetLoaderError;

use super::{core::SkeletonColliders, serial::SkeletonCollidersSerial};

#[derive(Default)]
pub struct SkeletonCollidersLoader;

impl AssetLoader for SkeletonCollidersLoader {
    type Asset = SkeletonColliders;
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
        let serial: SkeletonCollidersSerial = ron::de::from_bytes(&bytes)?;

        serial
            .to_value(load_context)
            .await
            .ok_or(AssetLoaderError::SkeletonColliderLoadError)
    }

    fn extensions(&self) -> &[&str] {
        &["coll.ron"]
    }
}
