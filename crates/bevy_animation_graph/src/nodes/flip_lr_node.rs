use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::flipping::flip_pose;
use crate::prelude::config::{FlipConfig, FlipConfigProxy};
use crate::prelude::{EditProxy, PassContext, SpecContext};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Default, NodeLike, Serialize, Deserialize)]
pub struct FlipLRNode {
    pub config: FlipConfig,
}

impl Default for FlipLRNode {
    fn default() -> Self {
        Self::new(FlipConfig::default())
    }
}

impl FlipLRNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(config: FlipConfig) -> Self {
        Self { config }
    }
}

impl NodeLike for FlipLRNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME, input);
        let in_pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;
        ctx.set_time(in_pose.timestamp);
        let Some(skeleton) = ctx.resources.skeleton_assets.get(&in_pose.skeleton) else {
            return Err(GraphError::SkeletonMissing(ctx.node_id()));
        };
        let flipped_pose = flip_pose(&in_pose, &self.config, skeleton);
        ctx.set_data_fwd(Self::OUT_POSE, flipped_pose);

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN_POSE.into(), DataSpec::Pose)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
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
pub struct FlipLRProxy {
    pub config: FlipConfigProxy,
}

impl EditProxy for FlipLRNode {
    type Proxy = FlipLRProxy;

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
    // TODO: Move serialization tests into "integration" tests (as they need to integrate with
    // Bevy's types). Test round-trip serialiation.
    // We create a Bevy type registry to test reflect-based serialization
    // #[test]
    // fn test_serialize() {
    //     let mut registry = TypeRegistry::new();
    //     registry.register::<FlipLRNode>();

    //     let node = super::FlipLRNode::default();
    //     let serializer = AnimationNodeSerializer {
    //         type_registry: &registry,
    //         name: "Test".to_string(),
    //         inner: Box::new(node),
    //     };
    //     let serialized = ron::to_string(&serializer).unwrap();
    //     assert_eq!(serialized, "(name:\"Test\",ty:\"bevy_animation_graph::nodes::flip_lr_node::FlipLRNode\",inner:(config:(name_mapper:Pattern((key_1:\"L\",key_2:\"R\",pattern_before:\"^.*\",pattern_after:\"$\")))))".to_string());
    // }

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
