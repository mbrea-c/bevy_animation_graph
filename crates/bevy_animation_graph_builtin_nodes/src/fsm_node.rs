use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::PinMap,
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
        todo!()
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

    fn data_input_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        let fsm = ctx
            .fsm_assets
            .get(&self.fsm)
            .unwrap_or_else(|| panic!("no FSM asset `{:?}`", self.fsm));
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
