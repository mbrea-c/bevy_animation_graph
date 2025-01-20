use bevy::prelude::*;
use bevy_animation_graph_editor::AnimationGraphEditorPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(AnimationGraphEditorPlugin);

    app.run();
}
