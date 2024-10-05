//! # Bevy Animation Graph
//!
//! **Bevy Animation Graph** provides a graph-based animation system for [Bevy](https://bevyengine.org/).
//!
//! ## Introduction
//!
//! There are three kinds of assets introduced by this library:
//! - [`GraphClip`], which are defined in `*.anim.ron` files. These assets contain animation data,
//!   similarly to Bevy's [`AnimationClip`]. The `*.anim.ron` files don't contain the actual
//!   animation data, but rather point to the source for the
//!   animation. Currently, animation from a Gltf file identified by their name label are supported.
//!   For example:
//!   ```ron
//!   (
//!       source: GltfNamed(
//!           // Asset path of the (root) gltf asset
//!           path: "models/Fox.glb",
//!           // Name of the animation within that asset
//!           animation_name: "Walk",
//!       ),
//!   )
//!   ```
//! - [`AnimationGraph`], defined in `*.animgraph.ron` files. These assets are the core
//!   of the library and specify the nodes, edges, inputs, outputs and default parameters of an
//!   animation graph. The animation player ([`AnimationGraphPlayer`]) uses a handle to an
//!   animation graph for playback, and can also pass inputs to the graph via input overlays.
//!   The preferred way of programmatically setting graph paramters is thus using the
//!   [`AnimationGraphPlayer`]'s API, as doing it this way will not actally mutate the graph.
//!   This enables the same graph to be used by multiple animation players at once.
//!   See [the examples section](#examples) for a developed example.
//!   The preferred way of editing graphs is using the visual editor: after [installing the
//!   editor](#editor-installation), run the command
//!   ```bash
//!   bevy_animation_graph_editor -a <PATH_TO_ASSETS_DIRECTORY>
//!   ```
//!   to start the editor on the given assets folder. At the moment the editor only supports
//!   creating and modifying animation graphs and state machines, but it can present a live preview of an
//!   `AnimatedScene` asset.
//! - [`StateMachine`], defined in `*.fsm.ron` files. These are used to define, as the name
//!   implies, _state machines_ where each state and transition plays back an animation graph and each transition's
//!   graph can query the source and target states' respective graphs (useful for blending in
//!   different ways, or playing a separate transition animation). State machines can be edited
//!   with the graphical editor and are used as nodes in an existing animation graph.
//! - [`AnimatedScene`], defined in `*.animscn.ron` files. These assets solve the ergonomics problem
//!   of spawning in a scene that is animated via an animation graph. Without [`AnimatedScene`],
//!   you would have to spawn the scene, manually find and remove remove Bevy's
//!   [`AnimationPlayer`], replace it with a [`AnimationGraphPlayer`] and set it to play a desired
//!   [`AnimationGraph`]. An `*.animscn.ron` file specifies a target scene file to spawn, the path
//!   to the [`AnimationPlayer`] to replace (using entity `Name`s) and the asset path of the
//!   animation graph to play. For example:
//!   ```ron
//!   (
//!       source: "models/Fox.glb#Scene0",
//!       path_to_player: ["root"],
//!       animation_graph: "animation_graphs/fox.animgraph.ron",
//!   )
//!   ```
//!   We can now simply instantiate an `AnimatedSceneBundle` with the given `AnimatedScene` handle,
//!   just like we would do with a regular scene:
//!   ```ignore
//!       //...
//!       commands.spawn(AnimatedSceneBundle {
//!           animated_scene: asset_server.load("animated_scenes/character.animscn.ron"),
//!           ..default()
//!       });
//!       //...
//!   ```
//!   Once the animated scene is finished successfully spawning, an `AnimatedSceneInstance` component
//!   will be added to it. For convenience, this component contains the entity id of the child containing
//!   the `AnimationGraphPlayer`, in case the user decides to manually set some animation graph parameters.
//!   If the animated scene spawning fails, (e.g. because the given `path_to_player` is incorrect),
//!   an error will be printed and the `AnimatedSceneFailed` component will be added instead.
//!
//! ## Nodes
//!
//! The currently implemented graph nodes are:
//! - [`ClipNode`]: Plays back an animation clip.
//! - [`ChainNode`]: Chains (plays one after the other) two animation inputs.
//! - [`BlendNode`]: Blends two animation inputs linearly based on an input factor.
//! - [`FlipLRNode`]: Mirrors an animation on the X axis, based on the bone names having `L` and `R`
//!   suffixes to specify which side they are on.
//! - [`LoopNode`]: Loops an animation input indefinitely.
//! - [`SpeedNode`]: Adjust the playback speed of an animation input.
//! - [`GraphNode`]: Nested animation graph. The node inputs and outputs match the nested graph's
//! - [`RotationNode`]: Applies a (quaternion) rotation to a set of bones from the input pose defined using a bone mask.
//!   inputs and outputs.
//! - Parameter arithmetic:
//!   - Floating point numbers (`f32`)
//!     - [`AddF32`]
//!     - [`SubF32`]
//!     - [`MulF32`]
//!     - [`DivF32`]
//!     - [`ClampF32`]
//!   - Vector (`vec3`)
//!     - [`RotationArcNode`]: Given two vectors, output quaternion rotation needed to rotate the first
//!       into the second.
//!
//! ## Editor installation
//!
//! The editor is in a separate crate, appropriately named `bevy_animation_graph_editor`. Install
//! it just like you would install any other cargo binary. In order to install the latest version
//! published to crates.io, run:
//! ```bash
//! cargo install bevy_animation_graph_editor
//! ```
//! To install the latest version from the git repository, run:
//! ```bash
//! cargo install --git 'https://github.com/mbrea-c/bevy_animation_graph.git' bevy_animation_graph_editor
//! ```
//! Finally, to install from a local version of the workspace, run
//! ```bash
//! cargo install --path <PATH_TO_WORKSPACE> bevy_animation_graph_editor
//! ```
//!
//! ## Examples
//!
//! ### Blend running and walking animation based on movement speed
//!
//! Consider the following simple scenario:
//!
//! - Inputs:
//!   - We have running and walking animations.
//!   - We have a target movement speed for the character.
//!   - We know the movement speeds corresponding to the unmodified walk and run
//!     animations, which we call `walk_base_speed` and `run_base_speed`.
//!   - We decide on a range of target speeds where the blend between walk and run
//!     should happen. We call this `blend_start` and `blend_end`.
//! - Desired output:
//!   - A animation that blends between the running and walking animation if the target speed is
//!     between `blend_start` and `blend_end`, and scales the playback speed to match the target
//!     speed based on `walk_base_speed` and `run_base_speed`.
//!
//! A solution to this problem is as follows:
//!
//! 1. The blend factor between the two animations can be computed as
//!
//!    ```text
//!    blend_fac = clamp((target_speed - blend_start) / (blend_end - blend_start), 0, 1)
//!    ```
//!
//!    The playback speed factor applied to both animations is then
//!
//!    ```text
//!    speed_fac = target_speed / (walk_base_speed * (1 - blend_fac) + run_base_speed * blend_fac)
//!    ```
//!
//! 2. Blend the two animations together using `blend_fac`. Loop the result and
//!    apply the speed factor `speed_fac`.
//!
//! The resulting graphs can be seen in the assets directory of [the source repository](https://github.com/mbrea-c/bevy_animation_graph), under
//! [assets/animation_graphs/velocity_to_params.animgraph.ron](https://github.com/mbrea-c/bevy_animation_graph/blob/master/assets/animation_graphs/velocity_to_params.animgraph.ron) (for computing `speed_fac` and `blend_fac`) and
//! [assets/animation_graphs/human.animgraph.ron](https://github.com/mbrea-c/bevy_animation_graph/blob/master/assets/animation_graphs/human.animgraph.ron)
//! (for the animation tasks).
//!
//!
//! [`ParamValue`]: crate::core::parameters::ParamValue
//! [`PassContext`]: crate::core::context::PassContext
//! [`AnimationNode`]: prelude::AnimationNode
//!
//! [`SpeedNode`]: crate::nodes::SpeedNode
//! [`ClipNode`]: crate::nodes::ClipNode
//! [`ChainNode`]: crate::nodes::ChainNode
//! [`BlendNode`]: crate::nodes::BlendNode
//! [`RotationNode`]: crate::nodes::RotationNode
//! [`FlipLRNode`]: crate::nodes::FlipLRNode
//! [`LoopNode`]: crate::nodes::LoopNode
//! [`GraphNode`]: crate::nodes::GraphNode
//! [`AddF32`]: crate::nodes::AddF32
//! [`SubF32`]: crate::nodes::SubF32
//! [`MulF32`]: crate::nodes::MulF32
//! [`DivF32`]: crate::nodes::DivF32
//! [`ClampF32`]: crate::nodes::ClampF32
//! [`RotationArcNode`]: crate::nodes::RotationArcNode
//!
//! [`NodeLike`]: crate::core::animation_node::NodeLike
//! [`GraphClip`]: crate::core::animation_clip::GraphClip
//! [`AnimationClip`]: bevy::animation::AnimationClip
//! [`AnimationPlayer`]: bevy::animation::prelude::AnimationPlayer
//! [`AnimationGraph`]: crate::core::animation_graph::AnimationGraph
//! [`AnimationGraphPlayer`]: crate::core::animation_graph_player::AnimationGraphPlayer
//! [`AnimatedScene`]: crate::core::animated_scene::AnimatedScene

pub mod core;
pub mod flipping;
pub mod interpolation;
pub mod node;
mod utils;

pub mod prelude {
    pub use super::core::prelude::*;
    pub use super::flipping::*;
    pub use super::interpolation::linear::*;
    pub use super::node::*;
    pub use super::utils::ordered_map::OrderedMap;
}
