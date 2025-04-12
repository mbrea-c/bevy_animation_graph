use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        system::{In, ResMut, Resource},
        world::World,
    },
    utils::HashMap,
};
use bevy_animation_graph::{
    core::animation_graph::{SourcePin, TargetPin},
    nodes::ClipNode,
    prelude::{AnimatedScene, AnimationGraph, AnimationNode, DataSpec, GraphClip},
};

use super::{run_handler, DynamicAction};

#[derive(Resource, Default)]
pub struct ClipPreviewScenes {
    pub previews: HashMap<AssetId<GraphClip>, Handle<AnimatedScene>>,
}

#[derive(Clone)]
pub struct CreateClipPreview {
    /// Clip to preview
    pub clip: Handle<GraphClip>,
    /// On what scene do you want to preview the clip? This will create a new scene that overrides
    /// its animation graph
    pub scene: Handle<AnimatedScene>,
}

impl DynamicAction for CreateClipPreview {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not create clip preview")(Self::system, *self)
    }
}

impl CreateClipPreview {
    pub fn system(
        In(action): In<Self>,
        mut previews: ResMut<ClipPreviewScenes>,
        mut graph_assets: ResMut<Assets<AnimationGraph>>,
        mut scene_assets: ResMut<Assets<AnimatedScene>>,
    ) {
        if previews.previews.contains_key(&action.clip.id()) {
            return;
        }

        let mut new_graph = AnimationGraph::new();
        new_graph.add_node(AnimationNode::new(
            "clip",
            Box::new(ClipNode::new(action.clip.clone(), None, None)),
        ));

        new_graph.add_output_parameter("pose", DataSpec::Pose);
        new_graph.add_output_time();

        new_graph.add_edge(
            SourcePin::NodeData("clip".into(), ClipNode::OUT_POSE.into()),
            TargetPin::OutputData("pose".into()),
        );
        new_graph.add_edge(SourcePin::NodeTime("clip".into()), TargetPin::OutputTime);

        let graph_handle = graph_assets.add(new_graph);

        let Some(mut scene) = scene_assets.get(&action.scene).cloned() else {
            return;
        };

        scene.processed_scene = None;
        scene.animation_graph = graph_handle;

        let scene_handle = scene_assets.add(scene);

        previews.previews.insert(action.clip.id(), scene_handle);
    }
}
