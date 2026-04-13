use bevy_animation_graph::core::{animation_graph::PinId, edge_data::DataSpecWithOptionalDefault};

use crate::ui::{
    reflect_lib::WidgetRegistry,
    reflect_widgets::{
        data_only_spec::DataOnlySpecReflectWidget, data_value::DataValueReflectWidget,
        hashmap::HashMapReflectWidget, ragdoll_config::RagdollConfigReflectWidget,
        vec3::Vec3ReflectWidget,
    },
};

pub mod data_only_spec;
pub mod data_value;
pub mod hashmap;
pub mod ragdoll_config;
pub mod vec3;

pub fn register_reflect_widgets(registry: &mut WidgetRegistry) {
    registry
        .add(HashMapReflectWidget::<String, String>::default())
        .add(DataOnlySpecReflectWidget::<
            PinId,
            DataSpecWithOptionalDefault,
        >::default())
        .add(DataValueReflectWidget::default())
        .add(Vec3ReflectWidget::default())
        .add(RagdollConfigReflectWidget::default());
}
