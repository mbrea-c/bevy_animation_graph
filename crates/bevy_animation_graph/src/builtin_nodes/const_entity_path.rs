use crate::core::animation_clip::EntityPath;
use crate::core::animation_graph::PinMap;
use crate::core::animation_node::NodeLike;
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::{DataSpec, DataValue};
use crate::core::errors::GraphError;
use bevy::reflect::Reflect;
use bevy::reflect::prelude::ReflectDefault;

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
