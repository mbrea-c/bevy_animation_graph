use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, io::Reader},
    gltf::Gltf,
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use super::GraphClip;
use crate::{errors::AssetLoaderError, event_track::EventTrack};

#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub enum GraphClipSource {
    GltfNamed {
        path: AssetPath<'static>,
        animation_name: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphClipSerial {
    pub source: GraphClipSource,
    pub skeleton: AssetPath<'static>,
    #[serde(default)]
    pub event_tracks: HashMap<String, EventTrack>,
}

#[derive(Default)]
pub struct GraphClipLoader;

impl AssetLoader for GraphClipLoader {
    type Asset = GraphClip;
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
        let serial: GraphClipSerial = ron::de::from_bytes(&bytes)?;

        let bevy_clip = match &serial.source {
            GraphClipSource::GltfNamed {
                path,
                animation_name,
            } => {
                let gltf_loaded_asset = load_context
                    .loader()
                    .immediate()
                    .with_unknown_type()
                    .load(path)
                    .await?;
                let gltf: &Gltf = gltf_loaded_asset.get().unwrap();

                let Some(clip_handle) = gltf
                    .named_animations
                    .get(&animation_name.clone().into_boxed_str())
                else {
                    return Err(AssetLoaderError::GltfMissingLabel(animation_name.clone()));
                };

                let Some(clip_path) = clip_handle.path() else {
                    return Err(AssetLoaderError::GltfMissingLabel(animation_name.clone()));
                };

                let clip_bevy: bevy::animation::AnimationClip = gltf_loaded_asset
                    .get_labeled(clip_path.label_cow().unwrap())
                    .unwrap()
                    .get::<bevy::animation::AnimationClip>()
                    .unwrap()
                    .clone();

                clip_bevy
            }
        };

        let skeleton = load_context.loader().load(serial.skeleton);

        let clip_mine = GraphClip::from_bevy_clip(
            bevy_clip,
            skeleton,
            serial.event_tracks,
            Some(serial.source.clone()),
        );

        Ok(clip_mine)
    }

    fn extensions(&self) -> &[&str] {
        &["anim.ron"]
    }
}

impl TryFrom<&GraphClip> for GraphClipSerial {
    type Error = ();

    fn try_from(value: &GraphClip) -> Result<Self, Self::Error> {
        let Some(source) = value.source.clone() else {
            return Err(());
        };

        Ok(Self {
            source,
            skeleton: value.skeleton.path().cloned().ok_or(())?,
            event_tracks: value.event_tracks.clone(),
        })
    }
}
