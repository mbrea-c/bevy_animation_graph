use std::any::{type_name_of_val, TypeId};

use crate::core::animation_clip::{GraphClip, Interpolation};
use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::id::BoneId;
use crate::core::pose::{BonePose, Pose};
use crate::core::prelude::{DataSpec, DataValue};
use crate::prelude::{PassContext, SpecContext};
use bevy::asset::Handle;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    AnimationNodeIndex, RotationCurveEvaluator, ScaleCurveEvaluator, TranslationCurveEvaluator,
    VariableCurve,
};
use bevy::reflect::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct ClipNode {
    pub(crate) clip: Handle<GraphClip>,
    pub(crate) override_duration: Option<f32>,
    pub(crate) override_interpolation: Option<Interpolation>,
}

impl ClipNode {
    pub const OUT_POSE: &'static str = "pose";
    pub fn new(
        clip: Handle<GraphClip>,
        override_duration: Option<f32>,
        override_interpolation: Option<Interpolation>,
    ) -> Self {
        Self {
            clip,
            override_duration,
            override_interpolation,
        }
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
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_duration_fwd(Some(self.clip_duration(&ctx)));
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let clip_duration = self.clip_duration(&ctx);

        let Some(clip) = ctx.resources.graph_clip_assets.get(&self.clip) else {
            // TODO: Should we propagate a GraphError instead?
            ctx.set_data_fwd(Self::OUT_POSE, DataValue::Pose(Pose::default()));
            return Ok(());
        };

        let prev_time = ctx.prev_time();
        let time_update = ctx.time_update_fwd()?;
        let time = time_update.apply(prev_time);

        ctx.set_time(time);

        let mut out_pose = Pose {
            timestamp: time,
            skeleton: clip.skeleton.clone(),
            ..Pose::default()
        };

        let time = time.clamp(0., clip_duration);

        for (bone_id, curves) in &clip.curves {
            let mut bone_pose = BonePose::default();
            for curve in curves {
                let value = sample_animation_curve(curve, time);
                match value {
                    CurveValue::Translation(t) => bone_pose.translation = Some(t),
                    CurveValue::Rotation(r) => bone_pose.rotation = Some(r),
                    CurveValue::Scale(s) => bone_pose.scale = Some(s),
                    CurveValue::BoneWeights(w) => bone_pose.weights = Some(w),
                }
            }
            out_pose.add_bone(bone_pose, BoneId::from(*bone_id));
        }

        ctx.set_data_fwd(Self::OUT_POSE, DataValue::Pose(out_pose));

        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "⏵ Animation Clip".into()
    }
}

enum CurveValue {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
    BoneWeights(Vec<f32>),
}

/// Sample the animation at a particular time
// HACK: We really need some API for sampling animation curves in Bevy
fn sample_animation_curve(curve: &VariableCurve, time: f32) -> CurveValue {
    let mut evaluator = curve.0.create_evaluator();
    let evaluator_type_id = evaluator.type_id();
    let evaluator_type_name = type_name_of_val(&evaluator);
    let node_index = AnimationNodeIndex::default();

    println!("Evaluator type name: {}", evaluator_type_name);

    curve
        .0
        .apply(evaluator.as_mut(), time, 1., node_index)
        .unwrap();

    if evaluator_type_id == TypeId::of::<TranslationCurveEvaluator>() {
        let t = evaluator
            .as_reflect()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("evaluator")
            .unwrap()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("stack")
            .unwrap()
            .reflect_ref()
            .as_list()
            .unwrap()
            .get(0)
            .unwrap()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("value")
            .unwrap()
            .try_downcast_ref::<Vec3>()
            .unwrap();
        CurveValue::Translation(*t)
    } else if evaluator_type_id == TypeId::of::<RotationCurveEvaluator>() {
        let r = evaluator
            .as_reflect()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("evaluator")
            .unwrap()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("stack")
            .unwrap()
            .reflect_ref()
            .as_list()
            .unwrap()
            .get(0)
            .unwrap()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("value")
            .unwrap()
            .try_downcast_ref::<Quat>()
            .unwrap();
        CurveValue::Rotation(*r)
    } else if evaluator_type_id == TypeId::of::<ScaleCurveEvaluator>() {
        let s = evaluator
            .as_reflect()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("evaluator")
            .unwrap()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("stack")
            .unwrap()
            .reflect_ref()
            .as_list()
            .unwrap()
            .get(0)
            .unwrap()
            .reflect_ref()
            .as_struct()
            .unwrap()
            .field("value")
            .unwrap()
            .try_downcast_ref::<Vec3>()
            .unwrap();
        CurveValue::Scale(*s)
    } else if evaluator_type_name.starts_with("WeightsCurveEvaluator") {
        todo!()
    } else {
        panic!("Evaluator type not supported!");
    }
}
