use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
    state_machine::{high_level::StateMachine, low_level::LowLevelStateMachine},
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct FSMNode {
    pub fsm: Handle<StateMachine>,
}

impl FSMNode {
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(fsm: Handle<StateMachine>) -> Self {
        Self { fsm }
    }
}

impl NodeLike for FSMNode {
    fn duration(&self, _ctx: NodeContext) -> Result<(), GraphError> {
        Err(GraphError::FsmDoesNotSupportDuration)
    }

    fn update(&self, ctx: NodeContext) -> Result<(), GraphError> {
        let fsm = ctx
            .graph_context
            .resources
            .state_machine_assets
            .get(&self.fsm)
            .unwrap();
        fsm.get_low_level_fsm().update(ctx)?;

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        let fsm = ctx
            .resources()
            .fsm_assets
            .get(&self.fsm)
            .ok_or(GraphError::FsmAssetMissing)?;

        ctx.set_from_node_spec(&fsm.node_spec);

        ctx.add_input_data(
            LowLevelStateMachine::DRIVER_EVENT_QUEUE,
            DataSpec::EventQueue,
        );

        Ok(())
    }

    fn display_name(&self) -> String {
        "⌘ FSM".into()
    }
}
