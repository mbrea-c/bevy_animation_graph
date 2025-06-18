use super::{
    Skeleton,
    serial::{SkeletonSerial, SkeletonSource},
};
use crate::core::{animation_clip::EntityPath, errors::AssetLoaderError};
use bevy::{
    animation::AnimationPlayer,
    asset::{AssetLoader, LoadContext, LoadedAsset, io::Reader},
    ecs::{hierarchy::Children, name::Name},
    gltf::Gltf,
    prelude::{Entity, With, World},
    scene::Scene,
};

#[derive(Default)]
pub struct SkeletonLoader;

impl AssetLoader for SkeletonLoader {
    type Asset = Skeleton;
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
        let serial: SkeletonSerial = ron::de::from_bytes(&bytes)?;
        let skeleton: Skeleton = match serial.source {
            SkeletonSource::Gltf { source, label } => {
                let gltf: LoadedAsset<Gltf> =
                    load_context.loader().immediate().load(source).await?;
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

#[allow(clippy::result_large_err)]
fn build_skeleton(world: &World) -> Result<Skeleton, AssetLoaderError> {
    let mut skeleton = Skeleton::default();

    let Some((root, root_name)) = find_root(world) else {
        return Err(AssetLoaderError::AnimatedSceneMissingRoot);
    };

    let mut query = world
        .try_query::<(Option<&Children>, &Name)>()
        .expect("This query should be readonly");

    skeleton.set_root(EntityPath::default().child(root_name).id());
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

fn find_root(world: &World) -> Option<(Entity, Name)> {
    let mut query = world
        .try_query_filtered::<(Entity, &Name), With<AnimationPlayer>>()
        .expect("This query should be readonly");

    let (entity, name) = query.iter(world).next()?;

    Some((entity, name.clone()))
}
