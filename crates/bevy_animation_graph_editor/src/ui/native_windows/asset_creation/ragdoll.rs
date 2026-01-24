use bevy_animation_graph::core::ragdoll::definition::Ragdoll;

use crate::ui::{
    native_windows::asset_creation::generic_with_path::CreateAssetFromPath,
    state_management::global::ragdoll::CreateRagdoll,
};

#[derive(Debug, Clone, Copy)]
pub struct CreateRagdollWindow;

impl CreateAssetFromPath for CreateRagdollWindow {
    const NAME: &'static str = "Create ragdoll";
    const EXTENSION: &'static str = ".rag.ron";

    fn create(&self, path: String, queue: &mut crate::ui::native_windows::OwnedQueue) {
        queue.trigger(CreateRagdoll {
            ragdoll: Ragdoll::default(),
            virtual_path: path.to_string(),
        });
    }
}
