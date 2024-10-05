use crate::{
    core::{
        animation_graph::PinMap,
        animation_node::{NodeLike, ReflectNodeLike},
        context::{PassContext, SpecContext},
        edge_data::DataSpec,
        errors::GraphError,
        state_machine::{high_level::StateMachine, low_level::LowLevelStateMachine},
    },
    utils::asset::GetTypedExt,
};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::graph"]
pub struct Fsm {
    pub fsm: Handle<StateMachine>,
}

impl Fsm {
    pub const OUT: &'static str = "pose";
}

impl NodeLike for Fsm {
    fn duration(&self, _ctx: PassContext) -> Result<(), GraphError> {
        todo!()
    }

    fn update(&self, ctx: PassContext) -> Result<(), GraphError> {
        let fsm = ctx
            .resources
            .state_machine_assets
            .get_typed(&self.fsm, &ctx.resources.loaded_untyped_assets)
            .unwrap();
        fsm.get_low_level_fsm().update(ctx)?;

        Ok(())
    }

    fn data_input_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        let fsm = ctx
            .fsm_assets
            .get_typed(&self.fsm, ctx.loaded_untyped_assets)
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
        [(Self::OUT.into(), DataSpec::Pose)].into()
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
