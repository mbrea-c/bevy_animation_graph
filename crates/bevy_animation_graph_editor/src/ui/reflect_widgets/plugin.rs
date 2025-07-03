use bevy::app::{App, Plugin};
use bevy_animation_graph::{
    core::{
        colliders::core::SkeletonColliders, event_track::TrackItemValue,
        state_machine::high_level::StateMachine,
    },
    prelude::{AnimatedScene, AnimationGraph, GraphClip},
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
        AssetPickerInspector::<SkeletonColliders>::default().register(app);
        TargetTracksInspector.register(app);
        SubmittableInspector::<String>::default().register(app);
        SubmittableInspector::<TrackItemValue>::default().register(app);
        Vec2PlaneInspector.register(app);
    }
}
