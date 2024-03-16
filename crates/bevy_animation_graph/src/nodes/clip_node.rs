use crate::core::animation_clip::{GraphClip, Keyframes};
use crate::core::animation_graph::TimeUpdate;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::pose::{BonePose, Pose, PoseSpec};
use crate::core::systems::get_keyframe;
use crate::prelude::{InterpolateLinear, PassContext, SpecContext};
use bevy::asset::Handle;
use bevy::reflect::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct ClipNode {
    pub(crate) clip: Handle<GraphClip>,
    pub(crate) override_duration: Option<f32>,
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
    pub fn clip_duration(&self, ctx: &PassContext) -> f32 {
        if let Some(duration) = self.override_duration {
            duration
        } else {
            ctx.resources
                .graph_clip_assets
                .get(&self.clip)
                .unwrap()
                .duration()
        }
    }
}

impl NodeLike for ClipNode {
    fn duration_pass(&self, ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(Some(Some(self.clip_duration(&ctx))))
    }

    fn pose_pass(
        &self,
        time_update: TimeUpdate,
        ctx: PassContext,
    ) -> Result<Option<Pose>, GraphError> {
        let clip_duration = self.clip_duration(&ctx);

        let Some(clip) = ctx.resources.graph_clip_assets.get(&self.clip) else {
            return Ok(Some(Pose::default()));
        };

        let prev_time = ctx.prev_time_fwd();
        let time = time_update.apply(prev_time);

        let mut out_pose = Pose::default();
        out_pose.timestamp = time;

        let time = time.clamp(0., clip_duration);

        for (path, bone_id) in &clip.paths {
            let curves = clip.get_curves(*bone_id).unwrap();
            let mut bone_pose = BonePose::default();
            for curve in curves {
                // Some curves have only one keyframe used to set a transform
                let keyframe_count = curve.keyframe_timestamps.len();

                // Find the current keyframe
                // PERF: finding the current keyframe can be optimised
                let (step_start, step_end, prev_is_wrapped, next_is_wrapped) = match curve
                    .keyframe_timestamps
                    .binary_search_by(|probe| probe.partial_cmp(&time).unwrap())
                {
                    // this curve is finished
                    Ok(n) if n >= curve.keyframe_timestamps.len() - 1 => (n, 0, false, true),
                    Ok(i) => (i, i + 1, false, false),
                    // this curve isn't started yet
                    Err(0) => (curve.keyframe_timestamps.len() - 1, 0, true, false),
                    // this curve is finished
                    Err(n) if n > curve.keyframe_timestamps.len() - 1 => (n - 1, 0, false, true),
                    Err(i) => (i - 1, i, false, false),
                };

                if prev_is_wrapped {
                    sample_one_keyframe(step_end, keyframe_count, &curve.keyframes, &mut bone_pose);
                    continue;
                }
                if next_is_wrapped {
                    sample_one_keyframe(
                        step_start,
                        keyframe_count,
                        &curve.keyframes,
                        &mut bone_pose,
                    );
                    continue;
                }
                if step_start == step_end {
                    sample_one_keyframe(
                        step_start,
                        keyframe_count,
                        &curve.keyframes,
                        &mut bone_pose,
                    );
                    continue;
                }

                let mut prev_timestamp = curve.keyframe_timestamps[step_start];
                let mut next_timestamp = curve.keyframe_timestamps[step_end];

                if prev_is_wrapped {
                    prev_timestamp -= clip_duration;
                } else if next_is_wrapped {
                    next_timestamp += clip_duration;
                }

                sample_two_keyframes(
                    step_start,
                    step_end,
                    prev_timestamp,
                    next_timestamp,
                    time,
                    keyframe_count,
                    &curve.keyframes,
                    &mut bone_pose,
                );
            }
            out_pose.add_bone(bone_pose, path.clone());
        }

        Ok(Some(out_pose))
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "⏵ Animation Clip".into()
    }
}

fn sample_two_keyframes(
    step_start: usize,
    step_end: usize,
    prev_timestamp: f32,
    next_timestamp: f32,
    time: f32,
    keyframe_count: usize,
    keyframes: &Keyframes,
    bone_pose: &mut BonePose,
) {
    let lerp = if next_timestamp == prev_timestamp {
        1.
    } else {
        (time - prev_timestamp) / (next_timestamp - prev_timestamp)
    };

    // Apply the keyframe
    match keyframes {
        Keyframes::Rotation(keyframes) => {
            let prev = keyframes[step_start];
            let mut next = keyframes[step_end];
            // Choose the smallest angle for the rotation
            if next.dot(prev) < 0.0 {
                next = -next;
            }

            bone_pose.rotation = Some(prev.interpolate_linear(&next, lerp));
        }
        Keyframes::Translation(keyframes) => {
            let prev = keyframes[step_start];
            let next = keyframes[step_end];

            bone_pose.translation = Some(prev.interpolate_linear(&next, lerp));
        }

        Keyframes::Scale(keyframes) => {
            let prev = keyframes[step_start];
            let next = keyframes[step_end];
            bone_pose.scale = Some(prev.interpolate_linear(&next, lerp));
        }

        Keyframes::Weights(keyframes) => {
            let target_count = keyframes.len() / keyframe_count;
            let morph_start: Vec<f32> = get_keyframe(target_count, keyframes, step_start).into();
            let morph_end: Vec<f32> = get_keyframe(target_count, keyframes, step_end).into();
            bone_pose.weights = Some(morph_start.interpolate_linear(&morph_end, lerp));
        }
    }
}

fn sample_one_keyframe(
    step: usize,
    keyframe_count: usize,
    keyframes: &Keyframes,
    bone_pose: &mut BonePose,
) {
    match keyframes {
        Keyframes::Rotation(keyframes) => {
            let frame = keyframes[step];

            bone_pose.rotation = Some(frame);
        }
        Keyframes::Translation(keyframes) => {
            let frame = keyframes[step];
            bone_pose.translation = Some(frame);
        }

        Keyframes::Scale(keyframes) => {
            let frame = keyframes[step];
            bone_pose.scale = Some(frame);
        }

        Keyframes::Weights(keyframes) => {
            let target_count = keyframes.len() / keyframe_count;
            let morph_start: Vec<f32> = get_keyframe(target_count, keyframes, step).into();
            bone_pose.weights = Some(morph_start);
        }
    }
}
