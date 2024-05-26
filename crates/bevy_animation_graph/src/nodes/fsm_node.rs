use crate::core::{
    animation_graph::PinMap,
    animation_node::{AnimationNode, AnimationNodeType, NodeLike},
    context::{PassContext, SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
    state_machine::{high_level::StateMachine, LowLevelStateMachine},
};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct FSMNode {
    pub fsm: Handle<StateMachine>,
}

impl FSMNode {
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(fsm: Handle<StateMachine>) -> Self {
        Self { fsm }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Fsm(self))
    }
}

impl NodeLike for FSMNode {
    fn duration(&self, _ctx: PassContext) -> Result<(), GraphError> {
        todo!()
    }

    fn update(&self, ctx: PassContext) -> Result<(), GraphError> {
        // TODO: Replace with graph error ?
        let fsm = ctx.resources.state_machine_assets.get(&self.fsm).unwrap();
        fsm.get_low_level_fsm().update(ctx)?;

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(
            LowLevelStateMachine::DRIVER_EVENT_QUEUE.into(),
            DataSpec::EventQueue,
        )]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _ctx: SpecContext) -> PinMap<()> {
        [].into()
    }

    fn time_output_spec(&self, _ctx: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "⌘ FSM".into()
    }
}
