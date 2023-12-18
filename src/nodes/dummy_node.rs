use crate::core::animation_graph::{OptParamSpec, ParamSpec, ParamValue, PinId, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, CustomNode, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::PoseFrame;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
pub struct DummyNode {}

impl DummyNode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(
            name.into(),
            AnimationNodeType::Custom(CustomNode::new(self)),
        )
    }
}

impl NodeLike for DummyNode {
    fn parameter_pass(&self, _: PassContext) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(&self, _: PassContext) -> Option<DurationData> {
        None
    }

    fn pose_pass(&self, _: TimeUpdate, _: PassContext) -> Option<PoseFrame> {
        None
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::new()
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::new()
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        false
    }

    fn display_name(&self) -> String {
        "Dummy".into()
    }
}
