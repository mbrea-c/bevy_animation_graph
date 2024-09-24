use super::{
    animation_graph::{PinId, PinMap},
    edge_data::DataSpec,
    errors::GraphError,
};
use crate::{
    nodes::{
        AbsF32, AddF32, BlendNode, BuildVec3Node, ChainNode, ClampF32, ClipNode, CompareF32,
        ConstBool, ConstEntityPath, ConstF32, ConstVec3Node, DecomposeVec3Node, DivF32, DummyNode,
        FSMNode, FireEventNode, FlipLRNode, FromEulerNode, GraphNode, IntoEulerNode,
        InvertQuatNode, LengthVec3Node, LerpVec3Node, LoopNode, MulF32, MulQuatNode,
        NormalizeVec3Node, PaddingNode, RotationArcNode, RotationNode, SelectF32, SlerpQuatNode,
        SpeedNode, SubF32, TwoBoneIKNode,
    },
    prelude::{PassContext, SpecContext},
};
use bevy::{reflect::prelude::*, utils::HashMap};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

pub trait NodeLike: Send + Sync + Reflect {
    fn duration(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn update(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        PinMap::new()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        PinMap::new()
    }

    fn time_input_spec(&self, _ctx: SpecContext) -> PinMap<()> {
        PinMap::new()
    }

    /// Specify whether or not a node outputs a pose, and which space the pose is in
    fn time_output_spec(&self, _ctx: SpecContext) -> Option<()> {
        None
    }

    /// The name of this node.
    fn display_name(&self) -> String;

    /// The order of the input pins. This way, you can mix time and data pins in the UI.
    fn input_pin_ordering(&self) -> PinOrdering {
        PinOrdering::default()
    }

    /// The order of the output pins. This way, you can mix time and data pins in the UI.
    fn output_pin_ordering(&self) -> PinOrdering {
        PinOrdering::default()
    }
}

#[derive(Clone, Reflect, Debug, Default)]
pub struct PinOrdering {
    keys: HashMap<PinId, usize>,
}

impl PinOrdering {
    pub fn new(keys: impl Into<HashMap<PinId, usize>>) -> Self {
        Self { keys: keys.into() }
    }

    pub fn pin_key(&self, pin_id: &PinId) -> usize {
        self.keys.get(pin_id).copied().unwrap_or(0)
    }
}

#[derive(Clone)]
pub struct CustomNode {
    pub node: Arc<Mutex<dyn NodeLike>>,
}

impl CustomNode {
    pub fn new(node: impl NodeLike + 'static) -> Self {
        Self {
            node: Arc::new(Mutex::new(node)),
        }
    }
}

impl Default for CustomNode {
    fn default() -> Self {
        Self {
            node: Arc::new(Mutex::new(DummyNode::new())),
        }
    }
}

impl std::fmt::Debug for CustomNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomNode")
    }
}

#[derive(Reflect, Clone, Debug, Default)]
pub struct AnimationNode {
    pub name: String,
    pub node: AnimationNodeType,
    #[reflect(ignore)]
    pub should_debug: bool,
}

impl AnimationNode {
    pub fn new_from_nodetype(name: String, node: AnimationNodeType) -> Self {
        Self {
            name,
            node,
            should_debug: false,
        }
    }
}

impl NodeLike for AnimationNode {
    fn duration(&self, ctx: PassContext) -> Result<(), GraphError> {
        self.node.map(|n| n.duration(ctx))
    }

    fn update(&self, ctx: PassContext) -> Result<(), GraphError> {
        self.node.map(|n| n.update(ctx))
    }

    fn data_input_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        self.node.map(|n| n.data_input_spec(ctx))
    }

    fn data_output_spec(&self, ctx: SpecContext) -> PinMap<DataSpec> {
        self.node.map(|n| n.data_output_spec(ctx))
    }

    fn time_input_spec(&self, ctx: SpecContext) -> PinMap<()> {
        self.node.map(|n| n.time_input_spec(ctx))
    }

    fn time_output_spec(&self, ctx: SpecContext) -> Option<()> {
        self.node.map(|n| n.time_output_spec(ctx))
    }

    fn display_name(&self) -> String {
        self.node.map(|n| n.display_name())
    }
}

#[derive(Reflect, Clone, Debug)]
#[reflect(Default)]
pub enum AnimationNodeType {
    // --- Dummy (default no-op node)
    // ------------------------------------------------
    Dummy(DummyNode),
    // ------------------------------------------------

    // --- Pose Nodes
    // ------------------------------------------------
    Clip(ClipNode),
    Chain(ChainNode),
    Blend(BlendNode),
    FlipLR(FlipLRNode),
    Loop(LoopNode),
    Padding(PaddingNode),
    Speed(SpeedNode),
    Rotation(RotationNode),
    // ------------------------------------------------

    // --- Pose space conversion
    // ------------------------------------------------
    // IntoBoneSpace(IntoBoneSpaceNode),
    // IntoCharacterSpace(IntoCharacterSpaceNode),
    // IntoGlobalSpace(IntoGlobalSpaceNode),
    // ExtendSkeleton(ExtendSkeleton),
    // ------------------------------------------------

    // --- IK space conversion
    // ------------------------------------------------
    TwoBoneIK(TwoBoneIKNode),
    // ------------------------------------------------

    // --- Constant nodes
    // ------------------------------------------------
    ConstBool(ConstBool),
    ConstF32(ConstF32),
    ConstVec3(ConstVec3Node),
    ConstEntityPath(ConstEntityPath),
    // ------------------------------------------------

    // --- F32 arithmetic nodes
    // ------------------------------------------------
    AddF32(AddF32),
    MulF32(MulF32),
    DivF32(DivF32),
    SubF32(SubF32),
    ClampF32(ClampF32),
    AbsF32(AbsF32),
    CompareF32(CompareF32),
    SelectF32(SelectF32),
    // ------------------------------------------------

    // --- Bool nodes
    // ------------------------------------------------
    // ------------------------------------------------

    // --- EventQueue nodes
    // ------------------------------------------------
    FireEvent(FireEventNode),
    // ------------------------------------------------

    // --- Vec3 arithmetic nodes
    // ------------------------------------------------
    RotationArc(RotationArcNode),
    BuildVec3(BuildVec3Node),
    DecomposeVec3(DecomposeVec3Node),
    LerpVec3(LerpVec3Node),
    NormalizeVec3(NormalizeVec3Node),
    LengthVec3(LengthVec3Node),
    // ------------------------------------------------

    // --- Quat arithmetic nodes
    // ------------------------------------------------
    SlerpQuat(SlerpQuatNode),
    FromEuler(FromEulerNode),
    IntoEuler(IntoEulerNode),
    MulQuat(MulQuatNode),
    InvertQuat(InvertQuatNode),
    // ------------------------------------------------
    Fsm(#[reflect(ignore)] FSMNode),
    // HACK: needs to be ignored for now due to:
    // https://github.com/bevyengine/bevy/issues/8965
    // Recursive reference causes reflection to fail
    Graph(#[reflect(ignore)] GraphNode),
    Custom(#[reflect(ignore)] CustomNode),
}

impl Default for AnimationNodeType {
    fn default() -> Self {
        Self::Dummy(DummyNode::new())
    }
}

impl AnimationNodeType {
    pub fn map<O, F>(&self, f: F) -> O
    where
        F: FnOnce(&dyn NodeLike) -> O,
    {
        match self {
            AnimationNodeType::Clip(n) => f(n),
            AnimationNodeType::Chain(n) => f(n),
            AnimationNodeType::Blend(n) => f(n),
            AnimationNodeType::FlipLR(n) => f(n),
            AnimationNodeType::Loop(n) => f(n),
            AnimationNodeType::Padding(n) => f(n),
            AnimationNodeType::Speed(n) => f(n),
            AnimationNodeType::Rotation(n) => f(n),
            AnimationNodeType::ConstF32(n) => f(n),
            AnimationNodeType::AddF32(n) => f(n),
            AnimationNodeType::MulF32(n) => f(n),
            AnimationNodeType::DivF32(n) => f(n),
            AnimationNodeType::SubF32(n) => f(n),
            AnimationNodeType::ClampF32(n) => f(n),
            AnimationNodeType::CompareF32(n) => f(n),
            AnimationNodeType::SelectF32(n) => f(n),
            AnimationNodeType::AbsF32(n) => f(n),
            AnimationNodeType::ConstBool(n) => f(n),
            AnimationNodeType::BuildVec3(n) => f(n),
            AnimationNodeType::DecomposeVec3(n) => f(n),
            AnimationNodeType::LerpVec3(n) => f(n),
            AnimationNodeType::SlerpQuat(n) => f(n),
            AnimationNodeType::FromEuler(n) => f(n),
            AnimationNodeType::IntoEuler(n) => f(n),
            AnimationNodeType::MulQuat(n) => f(n),
            AnimationNodeType::InvertQuat(n) => f(n),
            AnimationNodeType::RotationArc(n) => f(n),
            AnimationNodeType::Fsm(n) => f(n),
            AnimationNodeType::Graph(n) => f(n),
            // AnimationNodeType::IntoBoneSpace(n) => f(n),
            // AnimationNodeType::IntoCharacterSpace(n) => f(n),
            // AnimationNodeType::IntoGlobalSpace(n) => f(n),
            // AnimationNodeType::ExtendSkeleton(n) => f(n),
            AnimationNodeType::TwoBoneIK(n) => f(n),
            AnimationNodeType::FireEvent(n) => f(n),
            AnimationNodeType::Dummy(n) => f(n),
            AnimationNodeType::Custom(n) => f(n.node.lock().unwrap().deref()),
            AnimationNodeType::ConstEntityPath(n) => f(n),
            AnimationNodeType::NormalizeVec3(n) => f(n),
            AnimationNodeType::LengthVec3(n) => f(n),
            AnimationNodeType::ConstVec3(n) => f(n),
        }
    }

    pub fn map_mut<O, F>(&mut self, mut f: F) -> O
    where
        F: FnMut(&mut dyn NodeLike) -> O,
    {
        match self {
            AnimationNodeType::Clip(n) => f(n),
            AnimationNodeType::Chain(n) => f(n),
            AnimationNodeType::Blend(n) => f(n),
            AnimationNodeType::FlipLR(n) => f(n),
            AnimationNodeType::Loop(n) => f(n),
            AnimationNodeType::Padding(n) => f(n),
            AnimationNodeType::Speed(n) => f(n),
            AnimationNodeType::Rotation(n) => f(n),
            AnimationNodeType::ConstF32(n) => f(n),
            AnimationNodeType::AddF32(n) => f(n),
            AnimationNodeType::MulF32(n) => f(n),
            AnimationNodeType::DivF32(n) => f(n),
            AnimationNodeType::SubF32(n) => f(n),
            AnimationNodeType::ClampF32(n) => f(n),
            AnimationNodeType::CompareF32(n) => f(n),
            AnimationNodeType::SelectF32(n) => f(n),
            AnimationNodeType::AbsF32(n) => f(n),
            AnimationNodeType::ConstBool(n) => f(n),
            AnimationNodeType::BuildVec3(n) => f(n),
            AnimationNodeType::DecomposeVec3(n) => f(n),
            AnimationNodeType::LerpVec3(n) => f(n),
            AnimationNodeType::SlerpQuat(n) => f(n),
            AnimationNodeType::FromEuler(n) => f(n),
            AnimationNodeType::IntoEuler(n) => f(n),
            AnimationNodeType::MulQuat(n) => f(n),
            AnimationNodeType::InvertQuat(n) => f(n),
            AnimationNodeType::RotationArc(n) => f(n),
            AnimationNodeType::Fsm(n) => f(n),
            AnimationNodeType::Graph(n) => f(n),
            // AnimationNodeType::IntoBoneSpace(n) => f(n),
            // AnimationNodeType::IntoCharacterSpace(n) => f(n),
            // AnimationNodeType::IntoGlobalSpace(n) => f(n),
            // AnimationNodeType::ExtendSkeleton(n) => f(n),
            AnimationNodeType::TwoBoneIK(n) => f(n),
            AnimationNodeType::FireEvent(n) => f(n),
            AnimationNodeType::Dummy(n) => f(n),
            AnimationNodeType::Custom(n) => {
                let mut nod = n.node.lock().unwrap();
                f(nod.deref_mut())
            }
            AnimationNodeType::ConstEntityPath(n) => f(n),
            AnimationNodeType::NormalizeVec3(n) => f(n),
            AnimationNodeType::LengthVec3(n) => f(n),
            AnimationNodeType::ConstVec3(n) => f(n),
        }
    }

    pub fn inner_reflect(&mut self) -> &mut dyn Reflect {
        match self {
            AnimationNodeType::Clip(n) => n,
            AnimationNodeType::Chain(n) => n,
            AnimationNodeType::Blend(n) => n,
            AnimationNodeType::FlipLR(n) => n,
            AnimationNodeType::Loop(n) => n,
            AnimationNodeType::Padding(n) => n,
            AnimationNodeType::Speed(n) => n,
            AnimationNodeType::Rotation(n) => n,
            // AnimationNodeType::IntoBoneSpace(n) => n,
            // AnimationNodeType::IntoCharacterSpace(n) => n,
            // AnimationNodeType::IntoGlobalSpace(n) => n,
            // AnimationNodeType::ExtendSkeleton(n) => n,
            AnimationNodeType::TwoBoneIK(n) => n,
            AnimationNodeType::FireEvent(n) => n,
            AnimationNodeType::ConstF32(n) => n,
            AnimationNodeType::AddF32(n) => n,
            AnimationNodeType::MulF32(n) => n,
            AnimationNodeType::DivF32(n) => n,
            AnimationNodeType::SubF32(n) => n,
            AnimationNodeType::ClampF32(n) => n,
            AnimationNodeType::CompareF32(n) => n,
            AnimationNodeType::AbsF32(n) => n,
            AnimationNodeType::SelectF32(n) => n,
            AnimationNodeType::ConstBool(n) => n,
            AnimationNodeType::BuildVec3(n) => n,
            AnimationNodeType::DecomposeVec3(n) => n,
            AnimationNodeType::LerpVec3(n) => n,
            AnimationNodeType::SlerpQuat(n) => n,
            AnimationNodeType::FromEuler(n) => n,
            AnimationNodeType::IntoEuler(n) => n,
            AnimationNodeType::MulQuat(n) => n,
            AnimationNodeType::InvertQuat(n) => n,
            AnimationNodeType::RotationArc(n) => n,
            AnimationNodeType::Fsm(n) => n,
            AnimationNodeType::Graph(n) => n,
            AnimationNodeType::Dummy(n) => n,
            AnimationNodeType::Custom(_) => todo!(),
            AnimationNodeType::ConstEntityPath(n) => n,
            AnimationNodeType::NormalizeVec3(n) => n,
            AnimationNodeType::LengthVec3(n) => n,
            AnimationNodeType::ConstVec3(n) => n,
        }
    }
}
