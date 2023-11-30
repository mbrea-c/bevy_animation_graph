use crate::core::animation_clip::{GraphClip, Keyframes};
use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::{BoneFrame, PoseFrame, ValueFrame};
use crate::core::graph_context::{GraphContext, GraphContextTmp};
use crate::core::systems::get_keyframe;
use bevy::asset::Handle;
use bevy::reflect::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct ClipNode {
    clip: Handle<GraphClip>,
    override_duration: Option<f32>,
}

impl ClipNode {
    pub const OUTPUT: &'static str = "Pose Out";
    pub fn new(clip: Handle<GraphClip>, override_duration: Option<f32>) -> Self {
        Self {
            clip,
            override_duration,
        }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Clip(self))
    }

    #[inline]
    pub fn clip_duration(&self, context_tmp: &GraphContextTmp) -> f32 {
        if let Some(duration) = self.override_duration {
            duration
        } else {
            context_tmp
                .graph_clip_assets
                .get(&self.clip)
                .unwrap()
                .duration()
        }
    }
}

impl NodeLike for ClipNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<NodeInput, Option<f32>>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> Option<f32> {
        Some(self.clip_duration(context_tmp))
    }

    fn time_pass(
        &self,
        _input: TimeState,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        HashMap::new()
    }

    fn time_dependent_pass(
        &self,
        _inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let clip_duration = self.clip_duration(context_tmp);
        let clip = context_tmp.graph_clip_assets.get(&self.clip).unwrap();

        let time = context.get_times(name, path).unwrap();
        let timestamp = time.downstream.time;
        let time = timestamp.clamp(0., clip_duration);
        let mut animation_frame = PoseFrame::default();
        for (path, bone_id) in &clip.paths {
            let curves = clip.get_curves(*bone_id).unwrap();
            let mut frame = BoneFrame::default();
            for curve in curves {
                // Some curves have only one keyframe used to set a transform
                let keyframe_count = curve.keyframe_timestamps.len();

                // Find the current keyframe
                // PERF: finding the current keyframe can be optimised
                let (step_start, step_end) = match curve
                    .keyframe_timestamps
                    .binary_search_by(|probe| probe.partial_cmp(&time).unwrap())
                {
                    // this curve is finished
                    Ok(n) if n >= curve.keyframe_timestamps.len() - 1 => (n, 0),
                    Ok(i) => (i, i + 1),
                    // this curve isn't started yet
                    Err(0) => (0, 0),
                    // this curve is finished
                    Err(n) if n > curve.keyframe_timestamps.len() - 1 => (n - 1, 0),
                    Err(i) => (i - 1, i),
                };

                let prev_timestamp = curve.keyframe_timestamps[step_start];
                let mut next_timestamp = curve.keyframe_timestamps[step_end];

                let next_is_wrapped = if next_timestamp < prev_timestamp {
                    next_timestamp += clip_duration;
                    true
                } else {
                    false
                };

                // Apply the keyframe
                match &curve.keyframes {
                    Keyframes::Rotation(keyframes) => {
                        let prev = keyframes[step_start];
                        let mut next = keyframes[step_end];
                        // Choose the smallest angle for the rotation
                        if next.dot(prev) < 0.0 {
                            next = -next;
                        }

                        frame.rotation = Some(ValueFrame {
                            timestamp,
                            prev,
                            prev_timestamp,
                            next,
                            next_timestamp,
                            next_is_wrapped,
                        });
                    }
                    Keyframes::Translation(keyframes) => {
                        let prev = keyframes[step_start];
                        let next = keyframes[step_end];

                        frame.translation = Some(ValueFrame {
                            timestamp,
                            prev,
                            prev_timestamp,
                            next,
                            next_timestamp,
                            next_is_wrapped,
                        });
                    }

                    Keyframes::Scale(keyframes) => {
                        let prev = keyframes[step_start];
                        let next = keyframes[step_end];
                        frame.scale = Some(ValueFrame {
                            timestamp,
                            prev,
                            prev_timestamp,
                            next,
                            next_timestamp,
                            next_is_wrapped,
                        });
                    }

                    Keyframes::Weights(keyframes) => {
                        println!(
                            "Morph weight count: {:?} vs keyframe count: {:?}",
                            keyframes.len(),
                            keyframe_count
                        );
                        let target_count = keyframes.len() / keyframe_count;
                        let morph_start = get_keyframe(target_count, keyframes, step_start);
                        let morph_end = get_keyframe(target_count, keyframes, step_end);
                        frame.weights = Some(ValueFrame {
                            timestamp,
                            prev: morph_start.into(),
                            prev_timestamp,
                            next: morph_end.into(),
                            next_timestamp,
                            next_is_wrapped,
                        });
                    }
                }
            }
            animation_frame.add_bone(frame, path.clone());
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(animation_frame))])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::new()
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        return HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)]);
    }
}
