use super::animation_clip::Interpolation;
use super::animation_clip::loader::GraphClipLoader;
use super::edge_data::{AnimationEvent, EventQueue, SampledEvent};
use super::pose::Pose;
use super::prelude::loader::AnimatedSceneLoader;
use super::prelude::{GraphClip, locate_animated_scene_player};
use super::skeleton::Skeleton;
use super::skeleton::loader::SkeletonLoader;
use super::state_machine::high_level::GlobalTransition;
use super::systems::apply_animation_to_targets;
use super::{
    animated_scene::{AnimatedScene, spawn_animated_scenes},
    animation_graph::loader::AnimationGraphLoader,
    edge_data::{BoneMask, DataSpec, DataValue},
    state_machine::high_level::{StateMachine, loader::StateMachineLoader},
    systems::{animation_player, animation_player_deferred_gizmos},
};
use crate::core::colliders::core::ColliderLabel;

#[cfg(feature = "physics_avian")]
use crate::core::physics_systems_avian::{
    read_back_poses_avian, spawn_missing_ragdolls_avian, update_ragdoll_rigidbodies,
    update_ragdolls_avian,
};
use crate::core::ragdoll::bone_mapping::RagdollBoneMap;
use crate::core::ragdoll::bone_mapping_loader::RagdollBoneMapLoader;
use crate::core::ragdoll::definition::Ragdoll;
use crate::core::ragdoll::definition_loader::RagdollLoader;
use crate::nodes::blend_space_node::BlendSpaceNode;
use crate::nodes::const_ragdoll_config::ConstRagdollConfig;
use crate::nodes::{
    AbsF32, AddF32, BlendMode, BlendNode, BlendSyncMode, ChainNode, ClampF32, ClipNode, CompareF32,
    DivF32, DummyNode, FSMNode, FireEventNode, FlipLRNode, GraphNode, LoopNode, MulF32,
    PaddingNode, RotationArcNode, RotationNode, SpeedNode, SubF32, TwoBoneIKNode,
};
use crate::prelude::serial::SymmetryConfigSerial;
use crate::prelude::{AnimationGraph, AnimationGraphPlayer, config::SymmetryConfig};
use crate::{core::animation_clip::EntityPath, prelude::AnimationNode};
use bevy::ecs::intern::Interned;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use super::colliders::{core::SkeletonColliders, loader::SkeletonCollidersLoader};

/// Adds animation support to an app
pub struct AnimationGraphPlugin {
    physics_schedule: Interned<dyn ScheduleLabel>,
    final_schedule: Interned<dyn ScheduleLabel>,
}

impl Default for AnimationGraphPlugin {
    fn default() -> Self {
        Self {
            physics_schedule: FixedPostUpdate.intern(),
            final_schedule: PostUpdate.intern(),
        }
    }
}

impl AnimationGraphPlugin {
    pub fn from_physics_schedule(schedule: impl ScheduleLabel) -> Self {
        Self {
            physics_schedule: schedule.intern(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum AnimationGraphSet {
    /// This set runs in the same schedule as the physics update, before the physics update
    PrePhysics,
    /// This set runs in the same schedule as the physics update, after the physics update
    PostPhysics,
    /// This set runs at the end of the GPU frame (i.e. generally PostUpdate), it does not depend
    /// on physics update schedule.
    Final,
}

impl Plugin for AnimationGraphPlugin {
    fn build(&self, app: &mut App) {
        self.register_assets(app);
        self.register_types(app);
        self.register_nodes(app);
        self.register_component_hooks(app);

        app.configure_sets(
            self.physics_schedule,
            (
                AnimationGraphSet::PrePhysics,
                AnimationGraphSet::PostPhysics,
            )
                .chain(),
        );

        app.configure_sets(
            self.physics_schedule,
            AnimationGraphSet::Final.before(TransformSystems::Propagate),
        );

        #[cfg(feature = "physics_avian")]
        {
            use crate::core::physics_systems_avian::{
                update_relative_kinematic_body_velocities,
                update_relative_kinematic_position_based_body_velocities,
            };
            use avian3d::{
                dynamics::{
                    integrator::IntegrationSystems, solver::schedule::SubstepSolverSystems,
                },
                prelude::{PhysicsSchedule, PhysicsSystems, SolverSystems, SubstepSchedule},
            };

            app.configure_sets(
                self.physics_schedule,
                (
                    AnimationGraphSet::PrePhysics.before(PhysicsSystems::First),
                    AnimationGraphSet::PostPhysics.after(PhysicsSystems::Last),
                ),
            );

            app.add_systems(
                PhysicsSchedule,
                update_relative_kinematic_position_based_body_velocities
                    .after(SolverSystems::PreSubstep)
                    .before(SolverSystems::Substep),
            );

            app.add_systems(
                SubstepSchedule,
                update_relative_kinematic_body_velocities
                    .after(SubstepSolverSystems::SolveConstraints)
                    .before(IntegrationSystems::Position),
            );

            self.register_physics_types(app);
        }

        app.add_systems(PreUpdate, spawn_animated_scenes);

        app.add_systems(
            self.physics_schedule,
            (
                #[cfg(feature = "physics_avian")]
                spawn_missing_ragdolls_avian,
                animation_player,
                #[cfg(feature = "physics_avian")]
                update_ragdoll_rigidbodies,
                #[cfg(feature = "physics_avian")]
                update_ragdolls_avian,
            )
                .chain()
                .in_set(AnimationGraphSet::PrePhysics),
        );

        app.add_systems(
            self.physics_schedule,
            (
                #[cfg(feature = "physics_avian")]
                read_back_poses_avian,
            )
                .chain()
                .in_set(AnimationGraphSet::PostPhysics),
        );

        app.add_systems(
            self.final_schedule,
            (apply_animation_to_targets, animation_player_deferred_gizmos)
                .chain()
                .in_set(AnimationGraphSet::Final),
        );

        app.add_observer(locate_animated_scene_player);
    }
}

impl AnimationGraphPlugin {
    /// Registers asset types and their loaders
    fn register_assets(&self, app: &mut App) {
        app.init_asset::<GraphClip>()
            .init_asset_loader::<GraphClipLoader>()
            .register_asset_reflect::<GraphClip>();
        app.init_asset::<AnimationGraph>()
            .init_asset_loader::<AnimationGraphLoader>()
            .register_asset_reflect::<AnimationGraph>();
        app.init_asset::<AnimatedScene>()
            .init_asset_loader::<AnimatedSceneLoader>()
            .register_asset_reflect::<AnimatedScene>();
        app.init_asset::<StateMachine>()
            .init_asset_loader::<StateMachineLoader>()
            .register_asset_reflect::<StateMachine>();
        app.init_asset::<Skeleton>()
            .init_asset_loader::<SkeletonLoader>()
            .register_asset_reflect::<Skeleton>();
        app.init_asset::<SkeletonColliders>()
            .init_asset_loader::<SkeletonCollidersLoader>()
            .register_asset_reflect::<SkeletonColliders>();
        app.init_asset::<Ragdoll>()
            .init_asset_loader::<RagdollLoader>()
            .register_asset_reflect::<Ragdoll>();
        app.init_asset::<RagdollBoneMap>()
            .init_asset_loader::<RagdollBoneMapLoader>()
            .register_asset_reflect::<RagdollBoneMap>();
    }

    /// Registers built-in animation node implementations
    fn register_nodes(&self, app: &mut App) {
        app //
            .register_type::<ClipNode>()
            .register_type::<DummyNode>()
            .register_type::<ChainNode>()
            .register_type::<BlendNode>()
            .register_type::<BlendSpaceNode>()
            .register_type::<FlipLRNode>()
            .register_type::<GraphNode>()
            .register_type::<LoopNode>()
            .register_type::<PaddingNode>()
            .register_type::<RotationNode>()
            .register_type::<SpeedNode>()
            .register_type::<TwoBoneIKNode>()
            .register_type::<AbsF32>()
            .register_type::<AddF32>()
            .register_type::<ClampF32>()
            .register_type::<DivF32>()
            .register_type::<MulF32>()
            .register_type::<SubF32>()
            .register_type::<CompareF32>()
            .register_type::<FireEventNode>()
            .register_type::<RotationArcNode>()
            .register_type::<ConstRagdollConfig>()
            .register_type::<FSMNode>();
    }

    /// "Other" reflect registrations
    fn register_types(&self, app: &mut App) {
        app //
            .register_type::<Interpolation>()
            .register_type::<AnimationGraphPlayer>()
            .register_type::<EntityPath>()
            .register_type::<BoneMask>()
            .register_type::<Pose>()
            .register_type::<AnimationEvent>()
            .register_type::<SampledEvent>()
            .register_type::<EventQueue>()
            .register_type::<AnimationEvent>()
            .register_type::<SampledEvent>()
            .register_type::<DataValue>()
            .register_type::<DataSpec>()
            .register_type::<AnimationNode>()
            .register_type::<SymmetryConfig>()
            .register_type::<SymmetryConfigSerial>()
            .register_type::<BlendMode>()
            .register_type::<BlendSyncMode>()
            .register_type::<GlobalTransition>()
            .register_type::<ColliderLabel>()
            .register_type::<()>()
            .register_type_data::<(), ReflectDefault>();
    }

    #[cfg(feature = "physics_avian")]
    fn register_physics_types(&self, app: &mut App) {
        use crate::core::ragdoll::relative_kinematic_body::{
            RelativeKinematicBody, RelativeKinematicBodyPositionBased,
        };

        app.register_type::<RelativeKinematicBody>();
        app.register_type::<RelativeKinematicBodyPositionBased>();
    }

    fn register_component_hooks(&self, app: &mut App) {
        app.world_mut()
            .register_component_hooks::<AnimationGraphPlayer>()
            .on_replace(|mut world, context| {
                if let Some(spawned_ragdoll) = world
                    .entity(context.entity)
                    .get::<AnimationGraphPlayer>()
                    .and_then(|p| p.spawned_ragdoll.as_ref())
                    .map(|s| s.root)
                {
                    world.commands().entity(spawned_ragdoll).despawn();
                }
            });
    }
}
