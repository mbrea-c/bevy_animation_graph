use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        resource::Resource,
        system::{In, ResMut},
        world::World,
    },
    platform::collections::HashMap,
};
use bevy_animation_graph::{
    core::animation_graph::{NodeId, PinId, SourcePin, TargetPin},
    nodes::ClipNode,
    prelude::{AnimatedScene, AnimationGraph, AnimationNode, DataSpec, GraphClip},
};

use crate::ui::actions::ActionContext;

use super::{DynamicAction, run_handler};

#[derive(Resource, Default)]
pub struct ClipPreviewScenes {
    pub previews: HashMap<AssetId<GraphClip>, Handle<AnimatedScene>>,
}

#[derive(Resource, Default)]
pub struct NodePreviewScenes {
    pub previews: HashMap<NodePreviewKey, Handle<AnimatedScene>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodePreviewKey {
    pub graph: AssetId<AnimationGraph>,
    pub node_id: NodeId,
    pub pose_pin: PinId,
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
        let clip_node = AnimationNode::new("clip", ClipNode::new(action.clip.clone(), None, None));
        let clip_node_id = clip_node.id;
        new_graph.add_node(clip_node);

        new_graph.add_output_parameter("pose", DataSpec::Pose);
        new_graph.add_output_time();

        new_graph.add_edge(
            SourcePin::NodeData(clip_node_id, ClipNode::OUT_POSE.into()),
            TargetPin::OutputData("pose".into()),
        );
        new_graph.add_edge(SourcePin::NodeTime(clip_node_id), TargetPin::OutputTime);

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

#[derive(Clone)]
pub struct CreateTrackNodePreview {
    pub preview_key: NodePreviewKey,
    /// On what scene do you want to preview the clip? This will create a new scene that overrides
    /// its animation graph
    pub scene: Handle<AnimatedScene>,
}

impl DynamicAction for CreateTrackNodePreview {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not create clip preview")(Self::system, *self)
    }
}

impl CreateTrackNodePreview {
    pub fn system(
        In(action): In<Self>,
        mut previews: ResMut<NodePreviewScenes>,
        mut graph_assets: ResMut<Assets<AnimationGraph>>,
        mut scene_assets: ResMut<Assets<AnimatedScene>>,
    ) {
        if previews.previews.contains_key(&action.preview_key) {
            return;
        }

        let Some(existing_graph) = graph_assets.get(action.preview_key.graph) else {
            return;
        };

        let mut new_graph = existing_graph.clone();

        new_graph.add_output_parameter("pose", DataSpec::Pose);
        new_graph.add_output_time();

        new_graph.add_edge(
            SourcePin::NodeData(
                action.preview_key.node_id.clone(),
                action.preview_key.pose_pin.clone(),
            ),
            TargetPin::OutputData("pose".into()),
        );
        new_graph.add_edge(
            SourcePin::NodeTime(action.preview_key.node_id.clone()),
            TargetPin::OutputTime,
        );

        let graph_handle = graph_assets.add(new_graph);

        let Some(mut scene) = scene_assets.get(&action.scene).cloned() else {
            return;
        };

        scene.processed_scene = None;
        scene.animation_graph = graph_handle;

        let scene_handle = scene_assets.add(scene);

        previews.previews.insert(action.preview_key, scene_handle);
    }
}
