use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetPin<NodeId, PinId> {
    NodeParameter(NodeId, PinId),
    OutputParameter(PinId),
    NodePose(NodeId, PinId),
    OutputPose,
}

impl<NodeId: Eq, PinId> TargetPin<NodeId, PinId> {
    pub fn node_rename(&mut self, old_id: NodeId, new_id: NodeId) {
        match self {
            Self::NodeParameter(id, _) | Self::NodePose(id, _) => {
                if *id == old_id {
                    *id = new_id;
                }
            }
            _ => (),
        }
    }

    pub fn node_renamed(mut self, old_id: NodeId, new_id: NodeId) -> Self {
        self.node_rename(old_id, new_id);
        self
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourcePin<NodeId, PinId> {
    NodeParameter(NodeId, PinId),
    InputParameter(PinId),
    NodePose(NodeId),
    InputPose(PinId),
}

impl<NodeId: Eq, PinId> SourcePin<NodeId, PinId> {
    pub fn node_rename(&mut self, old_id: NodeId, new_id: NodeId) {
        match self {
            Self::NodeParameter(id, _) | Self::NodePose(id) => {
                if *id == old_id {
                    *id = new_id;
                }
            }
            _ => (),
        }
    }

    pub fn node_renamed(mut self, old_id: NodeId, new_id: NodeId) -> Self {
        self.node_rename(old_id, new_id);
        self
    }
}

// HACK: Until trait specialization is stabilized, we cannot implement this conversion
// using From trait. See tracking issue:
// https://github.com/rust-lang/rust/issues/31844

impl<N2, P2> TargetPin<N2, P2> {
    pub fn map_from<N1, P1>(value: TargetPin<N1, P1>) -> Self
    where
        N2: From<N1>,
        P2: From<P1>,
    {
        match value {
            TargetPin::NodeParameter(n, p) => Self::NodeParameter(N2::from(n), P2::from(p)),
            TargetPin::OutputParameter(p) => Self::OutputParameter(P2::from(p)),
            TargetPin::NodePose(n, p) => Self::NodePose(N2::from(n), P2::from(p)),
            TargetPin::OutputPose => Self::OutputPose,
        }
    }

    pub fn map_into<N1, P1>(self) -> TargetPin<N1, P1>
    where
        N1: From<N2>,
        P1: From<P2>,
    {
        TargetPin::map_from(self)
    }
}

impl<N2, P2> SourcePin<N2, P2> {
    pub fn map_from<N1, P1>(value: SourcePin<N1, P1>) -> Self
    where
        N2: From<N1>,
        P2: From<P1>,
    {
        match value {
            SourcePin::NodeParameter(n, p) => Self::NodeParameter(N2::from(n), P2::from(p)),
            SourcePin::InputParameter(p) => Self::InputParameter(P2::from(p)),
            SourcePin::NodePose(n) => Self::NodePose(N2::from(n)),
            SourcePin::InputPose(p) => Self::InputPose(P2::from(p)),
        }
    }

    pub fn map_into<N1, P1>(self) -> SourcePin<N1, P1>
    where
        N1: From<N2>,
        P1: From<P2>,
    {
        SourcePin::map_from(self)
    }
}
