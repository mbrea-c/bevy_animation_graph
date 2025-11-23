use bevy::{
    math::Quat,
    reflect::{Reflect, std_traits::ReflectDefault},
    transform::components::Transform,
};
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
    pose::BonePose,
};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
#[reflect(Default)]
pub enum RotationMode {
    #[default]
    Blend,
    Compose,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum RotationSpace {
    #[default]
    Local,
    Character,
    Global,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum ChainDecay {
    #[default]
    Linear,
}

#[derive(Reflect, Clone, Debug)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct RotationNode {
    pub application_mode: RotationMode,
    pub rotation_space: RotationSpace,
    pub chain_decay: ChainDecay,
    pub chain_length: usize,
    pub base_weight: f32,
}

impl Default for RotationNode {
    fn default() -> Self {
        Self {
            application_mode: RotationMode::Blend,
            rotation_space: RotationSpace::Local,
            chain_decay: ChainDecay::Linear,
            chain_length: 1,
            base_weight: 1.0,
        }
    }
}

impl RotationNode {
    pub const TARGET: &'static str = "bone_mask";
    pub const ROTATION: &'static str = "rotation";
    pub const IN_TIME: &'static str = "time";
    pub const IN_POSE: &'static str = "pose";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(
        mode: RotationMode,
        space: RotationSpace,
        chain_decay: ChainDecay,
        chain_length: usize,
        base_weight: f32,
    ) -> Self {
        Self {
            application_mode: mode,
            rotation_space: space,
            chain_decay,
            chain_length,
            base_weight,
        }
    }
}

impl NodeLike for RotationNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        // Pull incoming time update
        let input = ctx.time_update_fwd()?;
        // Push unchanged time update backwards.
        // We do this first to ensure that the time update is available for any other nodes that might need it
        ctx.set_time_update_back(Self::IN_TIME, input);

        let target = ctx.data_back(Self::TARGET)?.into_entity_path()?;
        let mut target = target.id();
        let rotation = ctx.data_back(Self::ROTATION)?.as_quat()?;
        let mut pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;
        let Some(skeleton) = ctx
            .graph_context
            .resources
            .skeleton_assets
            .get(&pose.skeleton)
        else {
            return Err(GraphError::SkeletonMissing(ctx.node_id.clone()));
        };

        if !pose.paths.contains_key(&target) {
            pose.add_bone(BonePose::default(), target);
        }

        // build bone chain
        let mut chain = vec![target];
        while let Some(parent) = skeleton.parent(&target) {
            if chain.len() >= self.chain_length {
                break;
            }

            chain.insert(0, parent);
            target = parent;
        }

        for (i, target) in chain.into_iter().enumerate() {
            let percent = (i + 1) as f32 / self.chain_length.max(1) as f32 * self.base_weight;
            let rotation_bone_space = match self.rotation_space {
                RotationSpace::Local => rotation,
                RotationSpace::Character => {
                    if let Some(parent) = skeleton.parent(&target) {
                        ctx.graph_context
                            .space_conversion()
                            .root_to_bone_space(
                                Transform::from_rotation(rotation),
                                &pose,
                                skeleton,
                                parent,
                            )
                            .rotation
                    } else {
                        rotation
                    }
                }
                RotationSpace::Global => {
                    if let Some(parent) = skeleton.parent(&target) {
                        ctx.graph_context
                            .space_conversion()
                            .global_to_bone_space(
                                Transform::from_rotation(rotation),
                                &pose,
                                skeleton,
                                parent,
                            )
                            .rotation
                    } else {
                        ctx.graph_context
                            .space_conversion()
                            .transform_global_to_character(
                                Transform::from_rotation(rotation),
                                skeleton,
                            )
                            .rotation
                    }
                }
            };

            let mut bone_pose = pose
                .paths
                .get(&target)
                .and_then(|bone_id| pose.bones.get_mut(*bone_id).cloned())
                .unwrap_or_default();

            if let Some(rot) = bone_pose.rotation {
                let rotation = match self.application_mode {
                    RotationMode::Blend => rot.slerp(rotation_bone_space, percent),
                    RotationMode::Compose => {
                        Quat::IDENTITY.slerp(rotation_bone_space, percent) * rot
                    }
                };
                bone_pose.rotation = Some(rotation);
            } else {
                bone_pose.rotation = Some(rotation_bone_space);
            }

            pose.add_bone(bone_pose, target);
        }

        ctx.set_time(pose.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, pose);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::TARGET, DataSpec::EntityPath)
            .add_input_data(Self::ROTATION, DataSpec::Quat)
            .add_input_data(Self::IN_POSE, DataSpec::Pose)
            .add_input_time(Self::IN_TIME);

        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "⮩ Rotation".into()
    }
}
