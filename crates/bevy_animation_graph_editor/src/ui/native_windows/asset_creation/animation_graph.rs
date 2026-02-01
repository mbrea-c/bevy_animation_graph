use bevy_animation_graph::core::animation_graph::AnimationGraph;

use crate::ui::{
    native_windows::asset_creation::generic_with_path::CreateAssetFromPath,
    state_management::global::animation_graph::CreateAnimationGraph,
};

#[derive(Debug, Clone, Copy)]
pub struct CreateAnimationGraphWindow;

impl CreateAssetFromPath for CreateAnimationGraphWindow {
    const NAME: &'static str = "Create animation graph";
    const EXTENSION: &'static str = ".animgraph.ron";

    fn create(&self, path: String, queue: &mut crate::ui::native_windows::OwnedQueue) {
        queue.trigger(CreateAnimationGraph {
            animation_graph: AnimationGraph::default(),
            virtual_path: path.to_string(),
        });
    }
}
