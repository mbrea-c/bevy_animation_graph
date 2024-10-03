use crate::core::{
    animation_graph::PinMap,
    animation_node::{AnimationNode, AnimationNodeType, NodeLike},
    context::{PassContext, SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
    state_machine::{high_level::StateMachine, low_level::LowLevelStateMachine},
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

    fn data_input_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        let fsm = ctx.fsm_assets.get(&self.fsm).unwrap();
        let fsm_args = fsm
            .input_data
            .iter()
            .map(|(pin_id, default_val)| (pin_id.clone(), DataSpec::from(default_val)));

        let mut input_map = PinMap::from([(
            LowLevelStateMachine::DRIVER_EVENT_QUEUE.into(),
            DataSpec::EventQueue,
        )]);
        input_map.extend(fsm_args);

        input_map
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
