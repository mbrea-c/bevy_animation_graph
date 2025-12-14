use bevy::{
    app::{App, Plugin, PreUpdate},
    asset::AssetApp,
    ecs::{
        intern::Interned,
        schedule::{IntoScheduleConfigs, ScheduleLabel, SystemSet},
    },
    reflect::prelude::ReflectDefault,
    transform::TransformSystems,
};

#[cfg(feature = "physics_avian")]
use crate::physics_systems_avian::{
    read_back_poses_avian, spawn_missing_ragdolls_avian, update_ragdoll_rigidbodies,
    update_ragdolls_avian,
};
use crate::{
    animated_scene::{
        AnimatedScene, loader::AnimatedSceneLoader, locate_animated_scene_player,
        spawn_animated_scenes,
    },
    animation_clip::{EntityPath, GraphClip, Interpolation, loader::GraphClipLoader},
    animation_graph::{AnimationGraph, loader::AnimationGraphLoader},
    animation_graph_player::AnimationGraphPlayer,
    animation_node::AnimationNode,
    edge_data::{
        DataSpec, DataValue,
        bone_mask::BoneMask,
        events::{AnimationEvent, EventQueue, SampledEvent},
    },
    pose::Pose,
    ragdoll::{
        bone_mapping::RagdollBoneMap, bone_mapping_loader::RagdollBoneMapLoader,
        definition::Ragdoll, definition_loader::RagdollLoader,
    },
    skeleton::{Skeleton, loader::SkeletonLoader},
    state_machine::high_level::{StateMachine, loader::StateMachineLoader},
    symmetry::{config::SymmetryConfig, serial::SymmetryConfigSerial},
    systems::{animation_player, animation_player_deferred_gizmos, apply_animation_to_targets},
};

/// Adds animation support to an app
pub struct AnimationGraphCorePlugin {
    pub physics_schedule: Interned<dyn ScheduleLabel>,
    pub final_schedule: Interned<dyn ScheduleLabel>,
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

impl Plugin for AnimationGraphCorePlugin {
    fn build(&self, app: &mut App) {
        self.register_assets(app);
        self.register_types(app);
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
            use avian3d::{
                dynamics::{
                    integrator::IntegrationSystems, solver::schedule::SubstepSolverSystems,
                },
                prelude::{PhysicsSchedule, PhysicsSystems, SolverSystems, SubstepSchedule},
            };

            use crate::physics_systems_avian::{
                update_relative_kinematic_body_velocities,
                update_relative_kinematic_position_based_body_velocities,
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

        #[cfg(feature = "physics_avian")]
        app.add_systems(
            self.physics_schedule,
            read_back_poses_avian
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

impl AnimationGraphCorePlugin {
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
        app.init_asset::<Ragdoll>()
            .init_asset_loader::<RagdollLoader>()
            .register_asset_reflect::<Ragdoll>();
        app.init_asset::<RagdollBoneMap>()
            .init_asset_loader::<RagdollBoneMapLoader>()
            .register_asset_reflect::<RagdollBoneMap>();
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
            .register_type::<()>()
            .register_type_data::<(), ReflectDefault>();
    }

    #[cfg(feature = "physics_avian")]
    fn register_physics_types(&self, app: &mut App) {
        use crate::ragdoll::relative_kinematic_body::{
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
