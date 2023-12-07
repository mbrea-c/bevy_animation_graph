//! # Bevy Animation Graph
//!
//! **Bevy Animation Graph** provides a graph-based animation system for [Bevy](https://bevyengine.org/).
//!
//! ## How does it work?
//!
//! Each node [`prelude::AnimationNode`] in an animation graph has an arbitrary number of inputs
//! and outputs, each with a particular type (defined in [`core::animation_graph::EdgeValue`]). Each edge
//! connects one output form one node to an input from another node.
//! There are two types inputs and otputs:
//! - **Parameters**: These inputs and outputs do not need to be sampled, that is, they are not
//! time-varying. The user can change them at will via graph inputs, but within a node the
//! parameter outputs only depend on parameter inputs. An example is the speed factor in a
//! [`prelude::SpeedNode`].
//! - **Time-dependent**: Conceptually, these outputs are *sampled* at a specific time. Given
//! a specific time query for a time-dependent output, the node computes appropriate time queries
//! for its time-dependent inputs. These new queries are then propagated backwards in the graph.
//! The results are propagated back upwards and produce the final time-dependent output.
//! These computations have access to previously used parameter inputs and outputs, declared node durations
//! and of course the queried times. Examples are the pose inputs and outputs from most nodes.
//!
//! In practice, the computation is performed in four passes rather than two:
//! 1. **Parameter pass**: Propagates parameter inputs and outputs as described above.
//! 2. **Duration pass**: Propagates node durations for time-dependent inputs and outputs. Each node
//! is given the durations of its time-dependent inputs and outputs the durations of its
//! time-dependent outputs. Durations can be affected by parameters (e.g. speed node has shorter
//! duration the higher the speed factor is), so it needs to happen after the parameter pass.
//! 3. **Time pass**: Propagates the time queries for time-dependent inputs and outputs, as
//!    described above. Durations can affect time query propagation (e.g. the duration of the
//!    inputs determines the time-query of a loop node), so it needs to happen after the duration
//!    pass.
//! 4. **Time-dependent pass**: Finally performs the time-dependent computations as described
//!    above.
//!
//! Each pass has access to the inputs and outputs of previous passes. In particular, the API is
//! described by the [`prelude::NodeLike`] trait, which every graph node type must implement.

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
