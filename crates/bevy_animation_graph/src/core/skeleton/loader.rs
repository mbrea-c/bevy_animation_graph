use super::Skeleton;
use crate::core::{animation_clip::EntityPath, errors::AssetLoaderError};
use bevy::{
    animation::AnimationPlayer,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext, LoadedAsset},
    core::Name,
    gltf::Gltf,
    hierarchy::Children,
    prelude::{Entity, With, World},
    scene::Scene,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SkeletonSerial {
    /// Path to animated scene source
    source: SkeletonSource,
}

#[derive(Serialize, Deserialize)]
enum SkeletonSource {
    Gltf { source: String, label: String },
}

#[derive(Default)]
pub struct SkeletonLoader;

impl AssetLoader for SkeletonLoader {
    type Asset = Skeleton;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let serial: SkeletonSerial = ron::de::from_bytes(&bytes)?;
        let skeleton: Skeleton = match serial.source {
            SkeletonSource::Gltf { source, label } => {
                let gltf: LoadedAsset<Gltf> = load_context.loader().direct().load(source).await?;
                let scn = gltf.get_labeled(label).unwrap().get::<Scene>().unwrap();
                build_skeleton(&scn.world)?
            }
        };

        Ok(skeleton)
    }

    fn extensions(&self) -> &[&str] {
        &["skn.ron"]
    }
}

fn build_skeleton(world: &World) -> Result<Skeleton, AssetLoaderError> {
    let mut skeleton = Skeleton::default();

    let Some(root) = find_root(world) else {
        return Err(AssetLoaderError::AnimatedSceneMissingRoot);
    };

    let mut query = unsafe {
        world
            .as_unsafe_world_cell_readonly()
            .world_mut()
            .query::<(Option<&Children>, &Name)>()
    };

    let mut pending_children: Vec<(Entity, EntityPath)> = vec![(root, EntityPath::default())];

    while let Some((cur_entity, parent_path)) = pending_children.pop() {
        let (maybe_children, cur_name) = query.get(world, cur_entity).unwrap();
        let cur_path = parent_path.child(cur_name.clone());

        skeleton.add_bone(cur_path.clone());

        if let Some(children) = maybe_children {
            for &child in children {
                pending_children.push((child, cur_path.clone()));
            }
        }
    }

    Ok(skeleton)
}

fn find_root(world: &World) -> Option<Entity> {
    let mut query = unsafe {
        world
            .as_unsafe_world_cell_readonly()
            .world_mut()
            .query_filtered::<Entity, With<AnimationPlayer>>()
    };

    let Some(entity) = query.iter(world).next() else {
        return None;
    };

    Some(entity)
}
