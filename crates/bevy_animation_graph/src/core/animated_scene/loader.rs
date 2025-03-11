use bevy::{
    asset::{io::Reader, AssetLoader, Handle, LoadContext},
    scene::Scene,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{errors::AssetLoaderError, skeleton::Skeleton},
    prelude::AnimationGraph,
};

use super::AnimatedScene;

#[derive(Serialize, Deserialize, Clone)]
struct AnimatedSceneSerial {
    source: String,
    animation_graph: String,
    skeleton: String,
    #[serde(default)]
    bone_path_overrides: HashMap<String, String>,
}

#[derive(Default)]
pub struct AnimatedSceneLoader;

impl AssetLoader for AnimatedSceneLoader {
    type Asset = AnimatedScene;
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
        let serial: AnimatedSceneSerial = ron::de::from_bytes(&bytes)?;

        let animation_graph: Handle<AnimationGraph> = load_context.load(serial.animation_graph);
        let skeleton: Handle<Skeleton> = load_context.load(serial.skeleton);
        let source: Handle<Scene> = load_context.load(serial.source);

        Ok(AnimatedScene {
            source,
            processed_scene: None,
            animation_graph,
            skeleton,
            bone_path_overrides: serial.bone_path_overrides,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animscn.ron"]
    }
}
