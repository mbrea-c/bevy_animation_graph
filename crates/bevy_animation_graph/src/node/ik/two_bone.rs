use crate::{
    core::{
        animation_clip::EntityPath,
        animation_graph::PinMap,
        animation_node::{NodeLike, ReflectNodeLike},
        errors::GraphError,
        pose::Pose,
        prelude::DataSpec,
        space_conversion::SpaceConversion,
    },
    prelude::{BoneDebugGizmos, PassContext, SpecContext},
    utils::{asset::GetTypedExt, unwrap::UnwrapVal},
};
use bevy::{
    color::LinearRgba,
    math::{Quat, Vec3},
    reflect::{std_traits::ReflectDefault, Reflect},
    transform::components::Transform,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::ik"]
pub struct TwoBone;

impl TwoBone {
    pub const IN_TIME: &'static str = "in_time";
    pub const IN_POSE: &'static str = "in_pose";
    pub const BONE_PATH: &'static str = "bone_path";
    pub const TARGET_POSITION: &'static str = "target_position";
    pub const OUT: &'static str = "out";
}

impl NodeLike for TwoBone {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME, input);
        let target: EntityPath = ctx.data_back(Self::BONE_PATH)?.val();
        let target = target.id();
        let target_pos_char: Vec3 = ctx.data_back(Self::TARGET_POSITION)?.val();
        //let targetrotation: Quat = ctx.parameter_back(Self::TARGETROT).unwrap();
        let mut pose: Pose = ctx.data_back(Self::IN_POSE)?.val();
        let Some(skeleton) = ctx
            .resources
            .skeleton_assets
            .get_typed(&pose.skeleton, &ctx.resources.loaded_untyped_assets)
        else {
            return Err(GraphError::SkeletonMissing(ctx.node_id()));
        };

        if let (Some(bone_id), Some(parent_path), Some(grandparent_path)) = (
            pose.paths.get(&target),
            skeleton.parent(&target),
            skeleton.parent(&target).and_then(|p| skeleton.parent(&p)),
        ) {
            // Debug render (if enabled)
            ctx.bone_gizmo(target, LinearRgba::RED, skeleton, Some(&pose));
            ctx.bone_gizmo(parent_path, LinearRgba::RED, skeleton, Some(&pose));

            let bone = pose.bones[*bone_id].clone();
            let target_gp = ctx.root_to_bone_space(
                Transform::from_translation(target_pos_char),
                &pose,
                skeleton,
                skeleton.parent(&grandparent_path).unwrap(),
            );

            let target_pos_gp = target_gp.translation;

            let parent_id = pose.paths.get(&parent_path).unwrap();
            let parent_transform = {
                let parent_bone = pose.bones.get_mut(*parent_id).unwrap();
                parent_bone.to_transform()
            };

            let grandparent_id = pose.paths.get(&grandparent_path).unwrap();
            let grandparent_bone = pose.bones.get_mut(*grandparent_id).unwrap();
            let grandparent_transform = grandparent_bone.to_transform();

            let bone_transform = bone.to_transform();

            let parent_gp_transform = grandparent_transform * parent_transform;
            let bone_gp_transform = parent_gp_transform * bone_transform;

            let (bone_gp_transform, parent_gp_transform, grandparent_transform) = two_bone_ik(
                bone_gp_transform,
                parent_gp_transform,
                grandparent_transform,
                target_pos_gp,
            );

            let parent_transform =
                Transform::from_matrix(grandparent_transform.compute_matrix().inverse())
                    * parent_gp_transform;
            let bone_transform =
                Transform::from_matrix(parent_gp_transform.compute_matrix().inverse())
                    * bone_gp_transform;

            pose.bones[*grandparent_id].rotation = Some(grandparent_transform.rotation);
            pose.bones[*parent_id].rotation = Some(parent_transform.rotation);
            pose.bones[*bone_id].rotation = Some(bone_transform.rotation);

            // Debug render (if enabled)
            ctx.bone_gizmo(target, LinearRgba::BLUE, skeleton, Some(&pose));
            ctx.bone_gizmo(parent_path, LinearRgba::BLUE, skeleton, Some(&pose));
        }
        ctx.set_time(pose.timestamp);
        ctx.set_data_fwd(Self::OUT, pose);
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::BONE_PATH.into(), DataSpec::EntityPath),
            (Self::TARGET_POSITION.into(), DataSpec::Vec3),
            (Self::IN_POSE.into(), DataSpec::Pose),
        ]
        .into()
    }

    fn display_name(&self) -> String {
        "Two Bone IK".into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _ctx: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _ctx: SpecContext) -> Option<()> {
        Some(())
    }
}

// Adapted from https://blog.littlepolygon.com/posts/twobone/
fn two_bone_ik(
    bone: Transform,
    parent: Transform,
    grandparent: Transform,
    target_pos: Vec3,
) -> (Transform, Transform, Transform) {
    const MAX_LEN_OFFSET: f32 = 0.01;

    // compute joint positions
    let in_end_loc = bone.translation;
    let in_mid_loc = parent.translation;
    let in_root_loc = grandparent.translation;

    // compute bone lengths
    let upper_len = in_root_loc.distance(in_mid_loc);
    let lower_len = in_mid_loc.distance(in_end_loc);
    let max_len = upper_len + lower_len - MAX_LEN_OFFSET;

    // compute input planar basis vectors
    let to_end = (in_end_loc - in_root_loc).normalize();
    let in_pole_vec = (in_mid_loc - in_root_loc).reject_from(to_end).normalize();

    // compute final planar basis vectors
    let to_target_offset = (target_pos - in_root_loc).clamp_length_max(max_len);
    let to_target_dist = to_target_offset.length();
    let to_target = to_target_offset / to_target_dist;

    let to_target_swing = Quat::from_rotation_arc(to_end, to_target);
    let out_pole_vec = to_target_swing * in_pole_vec;

    // apply law of cosines to get middle joint angle
    let denom = 2. * upper_len * to_target_dist;
    let mut cos_angle = 0.;
    if denom > f32::EPSILON {
        cos_angle = (to_target_dist * to_target_dist + upper_len * upper_len
            - lower_len * lower_len)
            / denom;
    }
    let angle = cos_angle.acos();

    // compute final joint positions
    let pole_dist = upper_len * angle.sin();
    let eff_dist = upper_len * cos_angle;
    let out_end_loc = in_root_loc + to_target_offset;
    let out_mid_loc = in_root_loc + eff_dist * to_target + pole_dist * out_pole_vec;

    // compute final rotations
    let in_to_mid = in_mid_loc - in_root_loc;
    let out_to_mid = out_mid_loc - in_root_loc;
    let root_swing = Quat::from_rotation_arc(in_to_mid.normalize(), out_to_mid.normalize());
    let in_end_loc_with_root_swing = in_root_loc + root_swing * (in_end_loc - in_root_loc);
    let to_in_end = in_end_loc_with_root_swing - out_mid_loc;
    let to_out_end = out_end_loc - out_mid_loc;
    let mid_swing =
        Quat::from_rotation_arc(to_in_end.normalize(), to_out_end.normalize()) * root_swing;

    // set up output transforms
    let out_grandparent = Transform {
        rotation: root_swing * grandparent.rotation,
        ..grandparent
    };

    let out_parent = Transform {
        translation: out_mid_loc,
        rotation: mid_swing * parent.rotation,
        ..parent
    };
    let out_bone = Transform {
        translation: out_end_loc,
        rotation: mid_swing * bone.rotation,
        ..bone
    };

    (out_bone, out_parent, out_grandparent)
}
