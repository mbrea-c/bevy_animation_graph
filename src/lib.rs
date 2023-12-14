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
//!           path: "models/character_rigged.gltf",
//!           // Name of the animation within that asset
//!           animation_name: "WalkBaked2",
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
//! - [`AnimatedScene`], defined in `*.animscn.ron` files. These assets solve the ergonomics problem
//!   of spawning in a scene that is animated via an animation graph. Without [`AnimatedScene`],
//!   you would have to spawn the scene, manually find and remove remove Bevy's
//!   [`AnimationPlayer`], replace it with a [`AnimationGraphPlayer`] and set it to play a desired
//!   [`AnimationGraph`]. An `*.animscn.ron` file specifies a target scene file to spawn, the path
//!   to the [`AnimationPlayer`] to replace (using entity `Name`s) and the asset path of the
//!   animation graph to play. For example:
//!   ```ron
//!   (
//!       source: "models/character_rigged.gltf#Scene0",
//!       path_to_player: ["Main Controller"],
//!       animation_graph: "animation_graphs/locomotion.animgraph.ron",
//!   )
//!   ```
//!   We can now simply instantiate an `AnimatedSceneBundle` with the given `AnimatedScene` handle,
//!   just like we would do with a regular scene:
//!   ```rust
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
//! ## How does it work?
//!
//! Each node [`AnimationNode`] in an animation graph has an arbitrary number of parameter inputs
//! and outputs, each with a particular type (defined in [`ParamValue`]). Each edge
//! connects one output form one node to an input from another node, that is, the mapping of
//! parameter inputs to parameter outputs is 1-many. Parameters are used for things like
//! the speed factor of an animation or the weight of a blend.
//!
//! Additionally, each node can have any number of pose inputs and a single pose output.
//! Pose edges connect each pose output to a single pose input, that is, the mapping of pose
//! outputs to pose inputs is 1-1.
//!
//! Conceptually, pose outputs are *sampled* at a specific time. Given
//! a specific time query for a pose output, the node computes appropriate time queries
//! for its pose inputs. These new queries are then propagated backwards in the graph.
//! The results are propagated back upwards and produce the final pose output.
//! These computations have access to previously used parameter inputs and outputs, declared node durations
//! and of course the queried times. Examples are the pose inputs and outputs from most nodes.
//!
//! In practice, the computation is performed in four passes rather than two:
//! 1. **Parameter pass**: Propagates parameter inputs and outputs as described above.
//! 2. **Duration pass**: Propagates node durations for pose inputs and outputs. Each node
//! is given the durations of its pose inputs and outputs the durations of its
//! pose outputs. Durations can be affected by parameters (e.g. speed node has shorter
//! duration the higher the speed factor is), so it needs to happen after the parameter pass.
//! 3. **Time pass**: Propagates the time queries for pose inputs and outputs, as
//!    described above. Durations can affect time query propagation (e.g. the duration of the
//!    inputs determines the time-query of a loop node), so it needs to happen after the duration
//!    pass.
//! 4. **Pose pass**: Finally performs the pose computations as described
//!    above.
//!
//! Each pass has access to the inputs and outputs of previous passes. In particular, the API is
//! described by the [`NodeLike`] trait, which every graph node type must implement.
//!
//!
//! [`ParamValue`]: crate::core::animation_graph::ParamValue
//! [`AnimationNode`]: prelude::AnimationNode
//! [`SpeedNode`]: prelude::SpeedNode
//! [`NodeLike`]: crate::core::animation_node::NodeLike
//! [`GraphClip`]: crate::core::animation_clip::GraphClip
//! [`AnimationClip`]: bevy::animation::AnimationClip
//! [`AnimationPlayer`]: bevy::animation::prelude::AnimationPlayer
//! [`AnimationGraph`]: crate::core::animation_graph::AnimationGraph
//! [`AnimationGraphPlayer`]: crate::core::animation_graph_player::AnimationGraphPlayer
//! [`AnimatedScene`]: crate::core::animated_scene::AnimatedScene

pub mod chaining;
pub mod core;
pub mod flipping;
pub mod interpolation;
pub mod nodes;
pub mod sampling;
mod utils;

pub mod prelude {
    pub use super::chaining::*;
    pub use super::core::prelude::*;
    pub use super::flipping::*;
    pub use super::interpolation::linear::*;
    pub use super::nodes::*;
    pub use super::sampling::prelude::*;
}
