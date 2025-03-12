use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, Handle, LoadContext},
    scene::Scene,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{errors::AssetLoaderError, skeleton::Skeleton},
    prelude::AnimationGraph,
};

use super::{AnimatedScene, Retargeting};

#[derive(Serialize, Deserialize, Clone)]
struct AnimatedSceneSerial {
    source: AssetPath<'static>,
    animation_graph: AssetPath<'static>,
    skeleton: AssetPath<'static>,
    #[serde(default)]
    retargeting: Option<RetargetingSerial>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RetargetingSerial {
    source_skeleton: AssetPath<'static>,
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
        let retargeting: Option<Retargeting> = serial.retargeting.map(|r| Retargeting {
            source_skeleton: load_context.load(r.source_skeleton),
            bone_path_overrides: r.bone_path_overrides,
        });

        Ok(AnimatedScene {
            source,
            processed_scene: None,
            animation_graph,
            skeleton,
            retargeting,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animscn.ron"]
    }
}
