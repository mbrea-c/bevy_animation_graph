use bevy::app::{App, Plugin};

use super::{
    checkbox::CheckboxInspector, entity_path::EntityPathInspector,
    pattern_mapper::PatternMapperInspector, EguiInspectorExtensionRegistration,
};
pub struct BetterInspectorPlugin;
impl Plugin for BetterInspectorPlugin {
    fn build(&self, app: &mut App) {
        EntityPathInspector.register(app);
        PatternMapperInspector.register(app);
        CheckboxInspector.register(app);
    }
}
