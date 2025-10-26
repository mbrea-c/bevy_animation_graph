use bevy::ecs::{
    component::Component, event::Event, observer::Trigger, query::With, system::Single,
};

use crate::ui::global_state::GlobalState;

#[derive(Debug, Component, Default, Clone, Hash)]
pub enum InspectorSelection {
    ActiveScene,

    ActiveFsm,
    ActiveFsmTransition,
    ActiveFsmState,

    ActiveGraph,
    ActiveNode,

    #[default]
    Nothing,
}

#[derive(Event)]
pub struct SetInspectorSelection {
    pub selection: InspectorSelection,
}

impl SetInspectorSelection {
    pub fn observe(
        set_inspector_selection: Trigger<SetInspectorSelection>,
        global_state: Single<&mut InspectorSelection, With<GlobalState>>,
    ) {
        *global_state.into_inner() = set_inspector_selection.event().selection.clone();
    }
}
