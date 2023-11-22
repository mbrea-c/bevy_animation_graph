use crate::animation::{
    get_keyframe, AnimationClip, AnimationNode, BoneFrame, EdgeSpec, EdgeValue, Keyframes,
    NodeInput, NodeLike, NodeOutput, PoseFrame, ValueFrame,
};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
pub struct ClipNode {
    clip: AnimationClip,
    override_duration: Option<f32>,
}

impl ClipNode {
    pub const OUTPUT: &'static str = "Pose";
    pub fn new(clip: AnimationClip, override_duration: Option<f32>) -> Self {
        Self {
            clip,
            override_duration,
        }
    }

    pub fn wrapped(self) -> AnimationNode {
        AnimationNode::Clip(self)
    }

    #[inline]
    pub fn clip_duration(&self) -> f32 {
        if let Some(duration) = self.override_duration {
            duration
        } else {
            self.clip.duration()
        }
    }
}

impl NodeLike for ClipNode {
    fn duration(&mut self, _input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        Some(self.clip_duration())
    }

    fn forward(&self, _time: f32) -> HashMap<NodeInput, f32> {
        HashMap::new()
    }

    fn backward(
        &self,
        time: f32,
        _inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let time = time.clamp(0., self.clip_duration());
        let mut animation_frame = PoseFrame::default();
        for (path, bone_id) in &self.clip.paths {
            let curves = self.clip.get_curves(*bone_id).unwrap();
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

                let ts_start = curve.keyframe_timestamps[step_start];
                let mut ts_end = curve.keyframe_timestamps[step_end];

                let next_is_wrapped = if ts_end < ts_start {
                    ts_end += self.clip_duration();
                    true
                } else {
                    false
                };

                // Apply the keyframe
                match &curve.keyframes {
                    Keyframes::Rotation(keyframes) => {
                        let rot_start = keyframes[step_start];
                        let mut rot_end = keyframes[step_end];
                        // Choose the smallest angle for the rotation
                        if rot_end.dot(rot_start) < 0.0 {
                            rot_end = -rot_end;
                        }

                        frame.rotation = Some(ValueFrame {
                            prev: rot_start,
                            prev_timestamp: ts_start,
                            next: rot_end,
                            next_timestamp: ts_end,
                            next_is_wrapped,
                        });
                    }
                    Keyframes::Translation(keyframes) => {
                        let translation_start = keyframes[step_start];
                        let translation_end = keyframes[step_end];

                        frame.translation = Some(ValueFrame {
                            prev: translation_start,
                            prev_timestamp: ts_start,
                            next: translation_end,
                            next_timestamp: ts_end,
                            next_is_wrapped,
                        });
                    }

                    Keyframes::Scale(keyframes) => {
                        let scale_start = keyframes[step_start];
                        let scale_end = keyframes[step_end];
                        frame.scale = Some(ValueFrame {
                            prev: scale_start,
                            prev_timestamp: ts_start,
                            next: scale_end,
                            next_timestamp: ts_end,
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
                            prev: morph_start.into(),
                            prev_timestamp: ts_start,
                            next: morph_end.into(),
                            next_timestamp: ts_end,
                            next_is_wrapped,
                        });
                    }
                }
            }
            animation_frame.add_bone(frame, path.clone());
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(animation_frame))])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        return HashMap::new();
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        return HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)]);
    }
}
