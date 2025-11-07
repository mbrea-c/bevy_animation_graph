use bevy::{asset::Assets, ecs::world::World};
use bevy_animation_graph::{
    core::{ragdoll::definition::Ragdoll, skeleton::Skeleton},
    nodes::const_ragdoll_config::ConstRagdollConfig,
};
use egui::Widget;

use crate::ui::{
    generic_widgets::ragdoll_config::RagdollConfigWidget,
    global_state::{
        active_ragdoll::ActiveRagdoll, active_skeleton::ActiveSkeleton, get_global_state,
    },
    node_editors::{Editable, NodeEditor},
};

pub struct RagdollConfigNodeEditor;

impl NodeEditor for RagdollConfigNodeEditor {
    type Target = ConstRagdollConfig;

    fn show(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        node: &mut Self::Target,
    ) -> egui::Response {
        let skeleton_assets = world.resource::<Assets<Skeleton>>();
        let ragdoll_assets = world.resource::<Assets<Ragdoll>>();
        let active_skeleton =
            get_global_state::<ActiveSkeleton>(world).and_then(|s| skeleton_assets.get(&s.handle));
        let active_ragdoll =
            get_global_state::<ActiveRagdoll>(world).and_then(|s| ragdoll_assets.get(&s.handle));
        RagdollConfigWidget::new_salted(&mut node.value, "Ragdoll config node editor")
            .with_skeleton(active_skeleton)
            .with_ragdoll(active_ragdoll)
            .ui(ui)
    }
}

impl Editable for ConstRagdollConfig {
    type Editor = RagdollConfigNodeEditor;

    fn get_editor(&self) -> Self::Editor {
        RagdollConfigNodeEditor
    }
}
