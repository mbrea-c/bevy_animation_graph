This example shows how to create your own editor binary using the provided
`AnimationGraphEditorPlugin`.

While the `bevy_animation_graph_editor` crate already bundles an editor binary, there
are cases where it may be preferable to create your own binary that uses the
editor as a plugin. For example:
* Your crate defines custom `AnimationNode`s, which you would like to register
  so that they are available in the editor.
* Your crate depends on a git version or fork of the animation graph workspace, and
  you would like to use that particular version of the editor without having to
  install it globally.

For demonstration purposes, we define a custom animation node in this crate,
which will appear in the editor.
