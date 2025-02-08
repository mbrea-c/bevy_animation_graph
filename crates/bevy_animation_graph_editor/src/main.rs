use bevy::prelude::*;
use bevy_animation_graph_editor::AnimationGraphEditorPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(AnimationGraphEditorPlugin)
        .add_plugins(WorldInspectorPlugin::new());

    app.run();
}
