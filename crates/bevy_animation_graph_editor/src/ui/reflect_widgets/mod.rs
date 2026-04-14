use bevy_animation_graph::core::{animation_graph::PinId, edge_data::DataSpecWithOptionalDefault};

use crate::ui::{
    reflect_lib::WidgetRegistry,
    reflect_widgets::{
        data_value::DataValueReflectWidget, hashmap::HashMapReflectWidget,
        ragdoll_config::RagdollConfigReflectWidget, sorted_map::SortedMapReflectWidget,
        vec3::Vec3ReflectWidget,
    },
};

pub mod data_value;
pub mod hashmap;
pub mod ragdoll_config;
pub mod sorted_map;
pub mod vec3;

pub fn register_reflect_widgets(registry: &mut WidgetRegistry) {
    registry
        .add(HashMapReflectWidget::<String, String>::default())
        .add(SortedMapReflectWidget::<PinId, DataSpecWithOptionalDefault>::default())
        .add(DataValueReflectWidget)
        .add(Vec3ReflectWidget)
        .add(RagdollConfigReflectWidget);
}
