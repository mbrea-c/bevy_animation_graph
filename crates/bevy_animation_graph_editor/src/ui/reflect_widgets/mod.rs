use crate::ui::{reflect_lib::WidgetRegistry, reflect_widgets::hashmap::HashMapReflectWidget};

pub mod hashmap;

pub fn register_reflect_widgets(registry: &mut WidgetRegistry) {
    registry.add(HashMapReflectWidget::<String, String>::default());
}
