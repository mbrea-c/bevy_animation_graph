extern crate bevy;
extern crate bevy_animation_graph;

use bevy::prelude::*;
use bevy_animation_graph::core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};
use bevy_animation_graph_editor::AnimationGraphEditorPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(AnimationGraphEditorPlugin);
    app.register_type::<MyCustomNode>();
    app.run();
}

/// Custom node that doesn't do anything productive, it randomizes the position of each joint
/// slightly
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct MyCustomNode;

impl MyCustomNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";
}

impl NodeLike for MyCustomNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        // Nodes need to "announce" their duration. Here, we just read the duration for our
        // dependency node and forward that, since this node does not change the length of the
        // clip. This won't always be the case; for example a node that applies a playback speed multiplier
        // to an animation will need to adjust the announced duration accordingly.
        let back_duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(back_duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        // Get time incoming time delta (or absolute time signal).
        let input = ctx.time_update_fwd()?;
        // Forward the time signal to dependency nodes
        ctx.set_time_update_back(Self::IN_TIME, input);
        // Now that we made the time signal available to dependency nodes, we can read their output
        // data. Nodes are evaluated lazily, so they won't compute anything until we attempt to
        // read them.
        let mut in_pose = ctx.data_back(Self::IN_POSE)?.into_pose().unwrap();

        // This node doesn't do anything "useful", but for demonstration purposes let's add some
        // random noise to the translation of each bone that has an animated translation.
        for bone in &mut in_pose.bones {
            if let Some(pos) = &mut bone.translation {
                let offset = Vec3::new(
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                ) * 0.035;

                *pos += offset;
            }
        }

        // Set the "current time" for this node.
        ctx.set_time(in_pose.timestamp);

        // Publish the output pose to the corresponding output data pin
        ctx.set_data_fwd(Self::OUT_POSE, in_pose);

        Ok(())
    }

    fn display_name(&self) -> String {
        // This is the name that will be displayed in the editor for the node
        "Custom example node".into()
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        // Specify input data pins for this node
        ctx.add_input_data(Self::IN_POSE, DataSpec::Pose);
        // Specify input time pins
        ctx.add_input_time(Self::IN_TIME);

        // Specify output data pins for this node
        ctx.add_output_data(Self::OUT_POSE, DataSpec::Pose);
        // Specify that this node has an output time pin. Nodes can only have one or zero output
        // time pins
        ctx.add_output_time();

        Ok(())
    }
}
