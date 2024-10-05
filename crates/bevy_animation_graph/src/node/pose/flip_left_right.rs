use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::flipping::flip_pose;
use crate::prelude::config::{FlipConfig, FlipConfigProxy};
use crate::prelude::{EditProxy, PassContext, SpecContext};
use crate::utils::asset::GetTypedExt;
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::pose"]
pub struct FlipLeftRight {
    pub config: FlipConfig,
}

impl FlipLeftRight {
    pub const IN_POSE: &str = "in_pose";
    pub const IN_TIME: &str = "in_time";
    pub const OUT: &str = "out";
}

impl NodeLike for FlipLeftRight {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME, input);
        let in_pose: Pose = ctx.data_back(Self::IN_POSE)?.val();
        ctx.set_time(in_pose.timestamp);
        let Some(skeleton) = ctx
            .resources
            .skeleton_assets
            .get_typed(&in_pose.skeleton, &ctx.resources.loaded_untyped_assets)
        else {
            return Err(GraphError::SkeletonMissing(ctx.node_id()));
        };
        let flipped_pose = flip_pose(&in_pose, &self.config, skeleton);
        ctx.set_data_fwd(Self::OUT, flipped_pose);

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN_POSE.into(), DataSpec::Pose)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "🚻 Flip Left/Right".into()
    }
}

#[derive(Clone, Reflect)]
pub struct FlipLeftRightProxy {
    pub config: FlipConfigProxy,
}

impl EditProxy for FlipLeftRight {
    type Proxy = FlipLeftRightProxy;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self {
        Self {
            // TODO: This will fail if the regex is incorrect, may cause some editor crashes
            config: proxy.config.clone().try_into().unwrap(),
        }
    }

    fn make_proxy(&self) -> Self::Proxy {
        Self::Proxy {
            config: FlipConfigProxy::from(self.config.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::animation_graph::serial::AnimationNodeSerializer;

    use super::*;
    use bevy::reflect::TypeRegistry;

    /// We create a Bevy type registry to test reflect-based serialization
    #[test]
    fn test_serialize() {
        let mut registry = TypeRegistry::new();
        registry.register::<FlipLeftRight>();

        let node = FlipLeftRight::default();
        let serializer = AnimationNodeSerializer {
            type_registry: &registry,
            name: "Test".to_string(),
            inner: Box::new(node),
        };
        let serialized = ron::to_string(&serializer).unwrap();
        assert_eq!(serialized, "(name:\"Test\",ty:\"bevy_animation_graph::nodes::flip_lr_node::FlipLRNode\",inner:(config:(name_mapper:Pattern((key_1:\"L\",key_2:\"R\",pattern_before:\"^.*\",pattern_after:\"$\")))))".to_string());
    }

    // TODO: How do we test deserialization?
    //
    // The main issue is that we need a LoadContext. I could not figure
    // out a way to mock it. Maybe we need to set up all the rigamarole
    // necessary for actually loading an animation graph, add the node under test
    // to an empty animation graph and test de/serialization on the graph
    // using real asset loaders?
    //
    // See: https://github.com/bevyengine/bevy/blob/7c6057bc69cd7263a2971d8653675a8c9c194710/crates/bevy_asset/src/server/loaders.rs#L333
}
