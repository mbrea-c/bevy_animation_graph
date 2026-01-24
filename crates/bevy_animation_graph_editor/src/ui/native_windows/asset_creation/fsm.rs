use bevy_animation_graph::core::state_machine::high_level::StateMachine;

use crate::ui::{
    native_windows::asset_creation::generic_with_path::CreateAssetFromPath,
    state_management::global::fsm::CreateFsm,
};

#[derive(Debug, Clone, Copy)]
pub struct CreateFsmWindow;

impl CreateAssetFromPath for CreateFsmWindow {
    const NAME: &'static str = "Create state machine";
    const EXTENSION: &'static str = ".fsm.ron";

    fn create(&self, path: String, queue: &mut crate::ui::native_windows::OwnedQueue) {
        queue.trigger(CreateFsm {
            fsm: StateMachine::default(),
            virtual_path: path.to_string(),
        });
    }
}
