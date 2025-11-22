use bevy::reflect::{Reflect, prelude::ReflectDefault};
use bevy_animation_graph_core::{
    animation_clip::EntityPath,
    animation_graph::PinMap,
    animation_node::NodeLike,
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{DataSpec, DataValue},
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct ConstEntityPath {
    pub path: EntityPath,
}

impl ConstEntityPath {
    pub const OUTPUT: &'static str = "out";

    pub fn new(path: EntityPath) -> Self {
        Self { path }
    }
}

impl NodeLike for ConstEntityPath {
    fn display_name(&self) -> String {
        "Entity Path".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, DataValue::EntityPath(self.path.clone()));
        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::EntityPath)].into()
    }
}
