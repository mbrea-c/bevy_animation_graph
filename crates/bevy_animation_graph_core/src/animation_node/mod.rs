pub mod dyn_node_like;
pub mod serial;

use std::{any::TypeId, fmt::Debug};

use bevy::{
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::{FromType, prelude::*},
};
use uuid::Uuid;

use crate::{
    animation_graph::{NodeId, PinId},
    animation_node::dyn_node_like::DynNodeLike,
    context::{
        new_context::NodeContext,
        spec_context::{NodeSpec, SpecContext, SpecResources},
    },
    errors::GraphError,
};

#[reflect_trait]
pub trait NodeLike: NodeLikeClone + Send + Sync + Debug + Reflect + 'static {
    #[allow(unused_variables)]
    fn duration(&self, ctx: NodeContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn update(&self, ctx: NodeContext) -> Result<(), GraphError>;
    fn spec(&self, ctx: SpecContext) -> Result<(), GraphError>;

    /// The name of this node.
    fn display_name(&self) -> String;
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
    pub id: NodeId,
    pub name: String,
    #[deref]
    pub inner: DynNodeLike,
    pub should_debug: bool,
}

impl AnimationNode {
    #[must_use]
    pub fn new(name: impl Into<String>, inner: impl NodeLike) -> Self {
        Self {
            name: name.into(),
            inner: DynNodeLike::new(inner),
            should_debug: false,
            id: NodeId(Uuid::new_v4()),
        }
    }

    pub fn new_spec(&self, resources: SpecResources) -> Result<NodeSpec, GraphError> {
        let mut spec = NodeSpec::default();
        let ctx = SpecContext::new(resources, &mut spec);
        self.spec(ctx)?;
        Ok(spec)
    }
}
