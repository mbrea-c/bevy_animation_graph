use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{EditProxy, NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use crate::symmetry::config::SymmetryConfig;
use crate::symmetry::flip_pose;
use crate::symmetry::serial::SymmetryConfigSerial;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Default, NodeLike, Serialize, Deserialize)]
pub struct FlipLRNode {
    pub config: SymmetryConfig,
}

impl Default for FlipLRNode {
    fn default() -> Self {
        Self::new(SymmetryConfig::default())
    }
}

impl FlipLRNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(config: SymmetryConfig) -> Self {
        Self { config }
    }
}

impl NodeLike for FlipLRNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME, input);
        let in_pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;
        ctx.set_time(in_pose.timestamp);
        let Some(skeleton) = ctx
            .graph_context
            .resources
            .skeleton_assets
            .get(&in_pose.skeleton)
        else {
            return Err(GraphError::SkeletonMissing(ctx.node_id.clone()));
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
    pub config: SymmetryConfigSerial,
}

impl EditProxy for FlipLRNode {
    type Proxy = FlipLRProxy;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self {
        Self {
            // TODO: This will fail if the regex is incorrect, may cause some editor crashes
            config: proxy.config.to_value().unwrap(),
        }
    }

    fn make_proxy(&self) -> Self::Proxy {
        Self::Proxy {
            config: SymmetryConfigSerial::from_value(&self.config),
        }
    }
}
