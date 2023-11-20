use bevy::utils::HashMap;

use crate::animation::{
    AnimationNode, EdgeSpec, EdgeValue, EntityPath, NodeInput, NodeOutput, NodeWrapper, Pose,
    PoseFrame,
};

pub struct FlipLRNode {}

impl FlipLRNode {
    pub const INPUT: &'static str = "Input Pose";
    pub const OUTPUT: &'static str = "Pose";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }
}

impl AnimationNode for FlipLRNode {
    fn duration(&mut self, input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        *input_durations.get(Self::INPUT.into()).unwrap()
    }

    fn forward(&self, time: f32) -> HashMap<NodeInput, f32> {
        HashMap::from([(Self::INPUT.into(), time)])
    }

    fn backward(
        &self,
        time: f32,
        inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        let mut flipped_pose_frame = PoseFrame {
            prev: Pose::default(),
            prev_timestamp: in_pose_frame.prev_timestamp,
            next: Pose::default(),
            next_timestamp: in_pose_frame.next_timestamp,
            next_is_wrapped: in_pose_frame.next_is_wrapped,
        };

        for (path, bone_id) in in_pose_frame.prev.paths {
            let mut channel = in_pose_frame.prev.channels[bone_id].clone();
            let new_path = EntityPath {
                parts: path
                    .parts
                    .iter()
                    .map(|part| {
                        let mut part = part.to_string();
                        if part.ends_with(" L") {
                            part = part.strip_suffix(" L").unwrap().into();
                            part.push_str(" R");
                        } else if part.ends_with(" R") {
                            part = part.strip_suffix(" R").unwrap().into();
                            part.push_str(" L");
                        }
                        part.into()
                    })
                    .collect(),
            };
            if channel.translation.is_some() {
                let translation = channel.translation.as_mut().unwrap();
                translation.x *= -1.;
            }
            if channel.rotation.is_some() {
                let mut rotation = channel.rotation.unwrap();
                rotation.x *= -1.;
                rotation.w *= -1.;
                channel.rotation = Some(rotation.normalize());
            }
            flipped_pose_frame.prev.add_channel(channel, new_path);
        }

        for (path, bone_id) in in_pose_frame.next.paths {
            let mut channel = in_pose_frame.next.channels[bone_id].clone();
            let new_path = EntityPath {
                parts: path
                    .parts
                    .iter()
                    .map(|part| {
                        let mut part = part.to_string();
                        if part.ends_with(" L") {
                            part = part.strip_suffix(" L").unwrap().into();
                            part.push_str(" R");
                        } else if part.ends_with(" R") {
                            part = part.strip_suffix(" R").unwrap().into();
                            part.push_str(" L");
                        }
                        part.into()
                    })
                    .collect(),
            };

            if channel.translation.is_some() {
                let translation = channel.translation.as_mut().unwrap();
                translation.x *= -1.;
            }
            if channel.rotation.is_some() {
                let mut rotation = channel.rotation.unwrap();
                rotation.x *= -1.;
                rotation.w *= -1.;
                channel.rotation = Some(rotation.normalize());
            }

            flipped_pose_frame.next.add_channel(channel, new_path);
        }

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(flipped_pose_frame),
        )])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::INPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
