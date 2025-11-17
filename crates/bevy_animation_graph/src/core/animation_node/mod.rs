pub mod dyn_node_like;

use super::{
    animation_graph::{PinId, PinMap},
    edge_data::DataSpec,
    errors::GraphError,
};
use crate::{
    nodes::DummyNode,
    prelude::{SpecContext, dyn_node_like::DynNodeLike, new_context::NodeContext},
};
use bevy::{
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::{FromType, prelude::*},
};
use std::{any::TypeId, fmt::Debug};

#[reflect_trait]
pub trait NodeLike: NodeLikeClone + Send + Sync + Debug + Reflect + 'static {
    fn duration(&self, _ctx: NodeContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn update(&self, _ctx: NodeContext) -> Result<(), GraphError> {
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
    fn input_pin_ordering(&self, _ctx: SpecContext) -> PinOrdering {
        PinOrdering::default()
    }

    /// The order of the output pins. This way, you can mix time and data pins in the UI.
    fn output_pin_ordering(&self, _ctx: SpecContext) -> PinOrdering {
        PinOrdering::default()
    }
}

pub trait NodeLikeClone {
    fn clone_node_like(&self) -> Box<dyn NodeLike>;
}

impl<T> NodeLikeClone for T
where
    T: 'static + NodeLike + Clone,
{
    fn clone_node_like(&self) -> Box<dyn NodeLike> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NodeLike> {
    fn clone(&self) -> Self {
        self.clone_node_like()
    }
}

#[derive(Clone)]
pub struct ReflectEditProxy {
    pub proxy_type_id: TypeId,
    pub from_proxy: fn(&dyn Reflect) -> Box<dyn NodeLike>,
    pub to_proxy: fn(&dyn NodeLike) -> Box<dyn Reflect>,
}

impl<T> FromType<T> for ReflectEditProxy
where
    T: EditProxy + NodeLike,
{
    fn from_type() -> Self {
        Self {
            proxy_type_id: TypeId::of::<<T as EditProxy>::Proxy>(),
            from_proxy: from_proxy::<T>,
            to_proxy: to_proxy::<T>,
        }
    }
}

fn from_proxy<T: EditProxy + NodeLike>(proxy: &dyn Reflect) -> Box<dyn NodeLike> {
    if proxy.type_id() == TypeId::of::<T::Proxy>() {
        let proxy = proxy.downcast_ref::<T::Proxy>().unwrap();
        Box::new(T::update_from_proxy(proxy))
    } else {
        panic!("Type mismatch")
    }
}

fn to_proxy<T: EditProxy + NodeLike>(node: &dyn NodeLike) -> Box<dyn Reflect> {
    if node.type_id() == TypeId::of::<T>() {
        let node = node.as_any().downcast_ref::<T>().unwrap();
        Box::new(T::make_proxy(node))
    } else {
        panic!("Type mismatch")
    }
}

pub trait EditProxy {
    type Proxy: Reflect + Clone;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self;
    fn make_proxy(&self) -> Self::Proxy;
}

#[derive(Clone, Reflect, Debug, Default)]
pub struct PinOrdering {
    keys: HashMap<PinId, i32>,
}

impl PinOrdering {
    pub fn new(keys: impl Into<HashMap<PinId, i32>>) -> Self {
        Self { keys: keys.into() }
    }

    pub fn pin_key(&self, pin_id: &PinId) -> i32 {
        self.keys.get(pin_id).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Reflect)]
pub struct AnimationNode {
    pub name: String,
    #[deref]
    pub inner: DynNodeLike,
    // #[reflect(ignore)] // manual reflect impl (see below)
    pub should_debug: bool,
}

impl AnimationNode {
    #[must_use]
    pub fn new(name: impl Into<String>, inner: Box<dyn NodeLike>) -> Self {
        Self {
            name: name.into(),
            inner: DynNodeLike(inner),
            should_debug: false,
        }
    }
}

impl Default for AnimationNode {
    fn default() -> Self {
        Self::new("", Box::new(DummyNode))
    }
}
