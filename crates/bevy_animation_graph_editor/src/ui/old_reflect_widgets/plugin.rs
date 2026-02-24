use bevy::app::{App, Plugin};
use bevy_animation_graph::core::{
    animated_scene::AnimatedScene,
    animation_clip::GraphClip,
    animation_graph::AnimationGraph,
    event_track::TrackItemValue,
    ragdoll::{bone_mapping::RagdollBoneMap, definition::Ragdoll},
    state_machine::high_level::StateMachine,
};

use super::{
    EguiInspectorExtensionRegistration, asset_picker::AssetPickerInspector,
    checkbox::CheckboxInspector, entity_path::EntityPathInspector,
    pattern_mapper::PatternMapperInspector, submittable::SubmittableInspector,
    target_tracks::TargetTracksInspector, vec2_plane::Vec2PlaneInspector,
};
pub struct BetterInspectorPlugin;
impl Plugin for BetterInspectorPlugin {
    fn build(&self, app: &mut App) {
        EntityPathInspector.register(app);
        PatternMapperInspector.register(app);
        CheckboxInspector.register(app);
        AssetPickerInspector::<AnimationGraph>::default().register(app);
        AssetPickerInspector::<StateMachine>::default().register(app);
        AssetPickerInspector::<GraphClip>::default().register(app);
        AssetPickerInspector::<AnimatedScene>::default().register(app);
        AssetPickerInspector::<Ragdoll>::default().register(app);
        AssetPickerInspector::<RagdollBoneMap>::default().register(app);
        TargetTracksInspector.register(app);
        SubmittableInspector::<String>::default().register(app);
        SubmittableInspector::<TrackItemValue>::default().register(app);
        Vec2PlaneInspector.register(app);
    }
}
