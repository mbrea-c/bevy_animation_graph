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
//! The resulting graph is defined like so:
//!
//! ```ron
//! // In file assets/animation_graphs/fox.animgraph.ron
//! (
//!     nodes: [
//!         (name: "Walk Clip", node: Clip("animations/fox_walk.anim.ron", None)),
//!         (name: "Run Clip",  node: Clip("animations/fox_run.anim.ron", None)),
//!         (name: "Blend", node: Blend),
//!         (name: "Loop Walk", node: Loop),
//!         (name: "Loop Run", node: Loop),
//!         (name: "Speed", node: Speed),
//!
//!         (name: "Param graph", node: Graph("animation_graphs/velocity_to_params.animgraph.ron")),
//!     ],
//!     input_parameters: {
//!         "Target Speed": F32(1.5),
//!         "Blend Start": F32(0.5),
//!         "Blend End": F32(1.5),
//!     },
//!     output_pose_spec: true,
//!     input_parameter_edges: [
//!         ("Target Speed", ("Param graph", "Target Speed")),
//!         ("Blend Start", ("Param graph", "Blend Start")),
//!         ("Blend End", ("Param graph", "Blend End")),
//!     ],
//!     output_pose_edge: Some("Speed"),
//!     parameter_edges: [
//!         (("Param graph", "blend_fac"),("Blend", "Factor")),
//!         (("Param graph", "speed_fac"),("Speed", "Speed")),
//!     ],
//!     pose_edges: [
//!         ("Walk Clip", ("Loop Walk", "Pose In")),
//!         ("Run Clip", ("Loop Run", "Pose In")),
//!         ("Loop Walk", ("Blend", "Pose In 1")),
//!         ("Loop Run", ("Blend", "Pose In 2")),
//!         ("Blend", ("Speed", "Pose In")),
//!     ],
//! )
//! ```
//!
//! We have extracted the computation of `blend_fac` and `speed_fac` into a separate
//! graph that we reference as a node above:
//!
//! ```ron
//! // In file: assets/animation_graphs/velocity_to_params.animgraph.ron
//! (
//!     nodes: [
//!         (name: "Alpha Tmp 1", node: SubF32),
//!         (name: "Alpha Tmp 2", node: SubF32),
//!         (name: "Alpha Tmp 3", node: DivF32),
//!         (name: "Target Speed Abs", node: AbsF32),
//!         (name: "Alpha", node: ClampF32),
//!
//!         (name: "1-Alpha", node: SubF32),
//!         (name: "Factored walk speed", node: MulF32),
//!         (name: "Factored run speed", node: MulF32),
//!         (name: "Blended base speed", node: AddF32),
//!         (name: "Speed factor", node: DivF32),
//!     ],
//!     input_parameters: {
//!         "Walk Base Speed": F32(0.3),
//!         "Run Base Speed": F32(0.8),
//!         "Blend Start": F32(1.0),
//!         "Blend End": F32(3.0),
//!
//!         "Target Speed": F32(1.5),
//!
//!         // Constant values
//!         "ZERO": F32(0.),
//!         "ONE": F32(1.),
//!     },
//!     output_parameter_spec: {
//!         "speed_fac": F32,
//!         "blend_fac": F32,
//!     },
//!     input_parameter_edges: [
//!         // Alpha clamp range
//!         ("ZERO", ("Alpha", "Min")),
//!         ("ONE", ("Alpha", "Max")),
//!
//!         // Alpha parameters
//!         ("Target Speed", ("Target Speed Abs", "F32 In")),
//!         ("Blend Start", ("Alpha Tmp 1", "F32 In 2")),
//!         ("Blend End",   ("Alpha Tmp 2", "F32 In 1")),
//!         ("Blend Start", ("Alpha Tmp 2", "F32 In 2")),
//!
//!         // Speed factor parameters
//!         ("ONE", ("1-Alpha", "F32 In 1")),
//!         ("Walk Base Speed", ("Factored walk speed", "F32 In 1")),
//!         ("Run Base Speed", ("Factored run speed", "F32 In 1")),
//!         ("Target Speed", ("Speed factor", "F32 In 1")),
//!     ],
//!     parameter_edges: [
//!         // Blend alpha computation
//!         // ((abs(target_speed) - blend_start) / (blend_end - blend_start)).clamp(0., 1.);
//!         (("Target Speed Abs", "F32 Out"), ("Alpha Tmp 1", "F32 In 1")),
//!         (("Alpha Tmp 1", "F32 Out"), ("Alpha Tmp 3", "F32 In 1")),
//!         (("Alpha Tmp 2", "F32 Out"), ("Alpha Tmp 3", "F32 In 2")),
//!         (("Alpha Tmp 3", "F32 Out"), ("Alpha", "F32 In")),
//!
//!         // Speed factor computation
//!         // target_speed / (walk_base_speed * (1. - alpha) + run_base_seed * alpha)
//!         (("Alpha", "F32 Out"),("1-Alpha", "F32 In 2")),
//!         (("1-Alpha", "F32 Out"),("Factored walk speed", "F32 In 2")),
//!         (("Alpha", "F32 Out"),("Factored run speed", "F32 In 2")),
//!         (("Factored walk speed", "F32 Out"), ("Blended base speed", "F32 In 1")),
//!         (("Factored run speed", "F32 Out"), ("Blended base speed", "F32 In 2")),
//!         (("Blended base speed", "F32 Out"),("Speed factor", "F32 In 2")),
//!     ],
//!     output_parameter_edges: [
//!         (("Alpha", "F32 Out"), "blend_fac"),
//!         (("Speed factor", "F32 Out"), "speed_fac"),
//!     ],
//! )
//! ```
//!
//! ## How does this library work?
//!
//! There are two types of values processed by the graph: parameters and poses.
//!
//! Each node [`AnimationNode`] in an animation graph has an arbitrary number of parameter inputs
//! and outputs, each with a particular type (defined in [`ParamValue`]). Each edge
//! connects one output from one node to an input from another node, that is, the mapping of
//! parameter inputs to parameter outputs is 1-many. Parameters are used for things like
//! the speed factor of an animation or the weight of a blend.
//!
//! Additionally, each node can have any number of pose inputs and a single pose output.
//! Pose edges connect each pose output to a single pose input, that is, the mapping of pose
//! outputs to pose inputs is 1-1. Nodes also must output the duration of the animation if they
//! have a pose output.
//!
//! Conceptually, pose outputs are *sampled* at a specific time, so when querying an input pose
//! nodes must provide a time update. This time update can be a time increment (delta) or an absolute time.
//!
//! Nodes query their inputs lazily using a provided graph context (rather than all inputs being
//! provided eagerly by the caller). The context will cache node outputs every frame to prevent repeated
//! computations. The API is described by the [`NodeLike`] trait, which every graph node type must implement.
//! The [`PassContext`] contains the API that a node has access to when called.
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
