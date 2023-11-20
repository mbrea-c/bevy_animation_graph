use bevy::utils::HashMap;

use crate::animation::{
    get_keyframe, AnimationClip, AnimationNode, ChannelPose, EdgeSpec, EdgeValue, Keyframes,
    NodeInput, NodeOutput, NodeWrapper, Pose, PoseFrame, WrapEnd,
};

pub struct ClipNode {
    clip: AnimationClip,
    override_duration: Option<f32>,
    override_timestamps: Option<Vec<f32>>,
    wrap_end: WrapEnd,
}

impl ClipNode {
    pub const OUTPUT: &'static str = "Pose";

    pub fn new(
        clip: AnimationClip,
        wrap_end: WrapEnd,
        override_duration: Option<f32>,
        override_timestamps: Option<Vec<f32>>,
    ) -> Self {
        Self {
            clip,
            wrap_end,
            override_duration,
            override_timestamps,
        }
    }

    pub fn wrapped(self) -> NodeWrapper {
        NodeWrapper::new(Box::new(self))
    }

    #[inline]
    pub fn clip_duration(&self) -> f32 {
        if let Some(duration) = self.override_duration {
            duration
        } else {
            self.clip.duration()
        }
    }

    fn pose_for_time(&self, time: f32) -> Pose {
        let time = time.clamp(0., self.clip_duration());
        let mut animation_pose = Pose::default();
        for (path, bone_id) in &self.clip.paths {
            let curves = self.clip.get_curves(*bone_id).unwrap();
            let mut pose = ChannelPose::default();
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
                    Ok(n) if n >= curve.keyframe_timestamps.len() - 1 => match self.wrap_end {
                        WrapEnd::Loop => (n, 0),
                        WrapEnd::Extend => (n, n),
                    },
                    Ok(i) => (i, i + 1),
                    // this curve isn't started yet
                    Err(0) => (0, 0),
                    // this curve is finished
                    Err(n) if n > curve.keyframe_timestamps.len() - 1 => match self.wrap_end {
                        WrapEnd::Loop => (n - 1, 0),
                        WrapEnd::Extend => (n - 1, n - 1),
                    },
                    Err(i) => (i - 1, i),
                };

                fn find_ts_lerp(time: f32, ts_start: f32, ts_end: f32, duration: f32) -> f32 {
                    if ts_end > ts_start {
                        (time - ts_start) / (ts_end - ts_start)
                    } else if ts_end < ts_start {
                        let out = (time - ts_start) / (ts_end + duration - ts_start);
                        if out.is_nan() {
                            0.
                        } else {
                            out
                        }
                    } else {
                        0.
                    }
                }

                let ts_start = curve.keyframe_timestamps[step_start];
                let ts_end = curve.keyframe_timestamps[step_end];
                let lerp = find_ts_lerp(time, ts_start, ts_end, self.clip_duration());

                // Apply the keyframe
                match &curve.keyframes {
                    Keyframes::Rotation(keyframes) => {
                        let rot_start = keyframes[step_start];
                        let mut rot_end = keyframes[step_end];
                        // Choose the smallest angle for the rotation
                        if rot_end.dot(rot_start) < 0.0 {
                            rot_end = -rot_end;
                        }

                        // Rotations are using a spherical linear interpolation
                        let rot = rot_start.normalize().slerp(rot_end.normalize(), lerp);
                        pose.rotation = Some(rot);
                    }
                    Keyframes::Translation(keyframes) => {
                        let translation_start = keyframes[step_start];
                        let translation_end = keyframes[step_end];
                        let result = translation_start.lerp(translation_end, lerp);
                        pose.translation = Some(result);
                    }

                    Keyframes::Scale(keyframes) => {
                        let scale_start = keyframes[step_start];
                        let scale_end = keyframes[step_end];
                        let result = scale_start.lerp(scale_end, lerp);
                        pose.scale = Some(result);
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
                        let result: Vec<f32> = morph_start
                            .iter()
                            .zip(morph_end)
                            .map(|(a, b)| *a + lerp * (*b - *a))
                            .collect();
                        pose.weights = Some(result);
                    }
                }
            }
            animation_pose.add_channel(pose, path.clone());
        }

        animation_pose
    }
}

impl AnimationNode for ClipNode {
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
        // Find closest keyframe timestamp range
        let mut prev_timestamp = 0.;
        let mut next_timestamp = self.clip_duration();
        let mut next_is_wrapped = false;

        if let Some(override_timestamps) = &self.override_timestamps {
            let keyframe_count = override_timestamps.len();
            let (step_start, step_end) = match override_timestamps
                .binary_search_by(|probe| probe.partial_cmp(&time).unwrap())
            {
                // this curve is finished
                Ok(n) if n >= keyframe_count - 1 => (n, 0),
                Ok(i) => (i, i + 1),
                // this curve isn't started yet
                Err(0) => (0, 0),
                // this curve is finished
                Err(n) if n > keyframe_count - 1 => (n - 1, 0),
                Err(i) => (i - 1, i),
            };

            let p_ts = override_timestamps[step_start];
            let mut n_ts = override_timestamps[step_end];

            if n_ts < p_ts {
                next_is_wrapped = true;
                n_ts += self.clip_duration();
            } else {
                next_is_wrapped = false;
            }

            if p_ts > prev_timestamp && p_ts < time {
                prev_timestamp = p_ts;
            }

            if n_ts < next_timestamp && n_ts > time {
                next_timestamp = n_ts;
            }
        } else {
            for (_, bone_id) in &self.clip.paths {
                let curves = self.clip.get_curves(*bone_id).unwrap();
                for curve in curves {
                    let keyframe_count = curve.keyframe_timestamps.len();
                    // Find the current keyframe
                    // PERF: finding the current keyframe can be optimised
                    let (step_start, step_end) = match curve
                        .keyframe_timestamps
                        .binary_search_by(|probe| probe.partial_cmp(&time).unwrap())
                    {
                        // this curve is finished
                        Ok(n) if n >= keyframe_count - 1 => (n, 0),
                        Ok(i) => (i, i + 1),
                        // this curve isn't started yet
                        Err(0) => (0, 0),
                        // this curve is finished
                        Err(n) if n > keyframe_count - 1 => (n - 1, 0),
                        Err(i) => (i - 1, i),
                    };

                    let p_ts = curve.keyframe_timestamps[step_start];
                    let mut n_ts = curve.keyframe_timestamps[step_end];

                    if n_ts < p_ts {
                        next_is_wrapped = true;
                        n_ts += self.clip_duration();
                    } else {
                        next_is_wrapped = false;
                    }

                    if p_ts > prev_timestamp && p_ts < time {
                        prev_timestamp = p_ts;
                    }

                    if n_ts < next_timestamp && n_ts > time {
                        next_timestamp = n_ts;
                    }
                }
            }
        }

        let prev_pose = self.pose_for_time(prev_timestamp);
        let next_pose = self.pose_for_time(next_timestamp);

        HashMap::from([(
            Self::OUTPUT.into(),
            EdgeValue::PoseFrame(PoseFrame {
                prev: prev_pose,
                next: next_pose,
                prev_timestamp,
                next_timestamp,
                next_is_wrapped,
            }),
        )])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        return HashMap::new();
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        return HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)]);
    }
}
