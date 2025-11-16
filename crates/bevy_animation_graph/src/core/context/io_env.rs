use std::borrow::Cow;

use bevy::{platform::collections::HashMap, reflect::Reflect};

use crate::{
    core::{
        animation_graph::{PinId, SourcePin, TargetPin, TimeUpdate},
        duration_data::DurationData,
        errors::GraphError,
    },
    prelude::{DataValue, new_context::GraphContext},
};

pub trait GraphIoEnv {
    fn get_data_back(&self, pin_id: PinId, ctx: GraphContext) -> Result<DataValue, GraphError>;
    fn get_duration_back(
        &self,
        pin_id: PinId,
        ctx: GraphContext,
    ) -> Result<DurationData, GraphError>;
    fn get_time_fwd(&self, ctx: GraphContext) -> Result<TimeUpdate, GraphError>;
}

pub struct GraphIoEnvBox<'a> {
    value: &'a dyn GraphIoEnv,
}

impl<'a> GraphIoEnvBox<'a> {
    pub fn new(value: &'a dyn GraphIoEnv) -> Self {
        Self { value }
    }
}

impl<'a> Clone for GraphIoEnvBox<'a> {
    fn clone(&self) -> Self {
        Self { value: self.value }
    }
}

impl<'a> GraphIoEnv for GraphIoEnvBox<'a> {
    fn get_data_back(&self, pin_id: PinId, ctx: GraphContext) -> Result<DataValue, GraphError> {
        self.value.get_data_back(pin_id, ctx)
    }

    fn get_duration_back(
        &self,
        pin_id: PinId,
        ctx: GraphContext,
    ) -> Result<DurationData, GraphError> {
        self.value.get_duration_back(pin_id, ctx)
    }

    fn get_time_fwd(&self, ctx: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.value.get_time_fwd(ctx)
    }
}

#[derive(Clone)]
pub struct EmptyIoEnv;

impl GraphIoEnv for EmptyIoEnv {
    fn get_data_back(&self, pin_id: PinId, _: GraphContext) -> Result<DataValue, GraphError> {
        Err(GraphError::OutputMissing(SourcePin::InputData(pin_id)))
    }

    fn get_duration_back(
        &self,
        pin_id: PinId,
        _: GraphContext,
    ) -> Result<DurationData, GraphError> {
        Err(GraphError::DurationMissing(SourcePin::InputTime(pin_id)))
    }

    fn get_time_fwd(&self, _: GraphContext) -> Result<TimeUpdate, GraphError> {
        Err(GraphError::TimeUpdateMissingBack(TargetPin::OutputTime))
    }
}
#[derive(Clone, Reflect, Default)]
pub struct IoOverrides {
    pub data: HashMap<PinId, DataValue>,
    pub duration: HashMap<PinId, DurationData>,
    pub time: Option<TimeUpdate>,
}

impl IoOverrides {
    pub fn clear(&mut self) {
        self.data.clear();
        self.duration.clear();
        self.time = None;
    }
}

#[derive(Clone)]
pub struct OverrideIoEnv<'a, T: ToOwned> {
    pub overrides: Cow<'a, IoOverrides>,
    pub inner: Cow<'a, T>,
}

impl<'a, T: GraphIoEnv + Clone> GraphIoEnv for OverrideIoEnv<'a, T> {
    fn get_data_back(&self, pin_id: PinId, ctx: GraphContext) -> Result<DataValue, GraphError> {
        self.overrides
            .data
            .get(&pin_id)
            .cloned()
            .ok_or_else(|| GraphError::OutputMissing(SourcePin::InputData(pin_id.clone())))
            .or(self.inner.get_data_back(pin_id, ctx))
    }

    fn get_duration_back(
        &self,
        pin_id: PinId,
        ctx: GraphContext,
    ) -> Result<DurationData, GraphError> {
        self.overrides
            .duration
            .get(&pin_id)
            .cloned()
            .ok_or_else(|| GraphError::DurationMissing(SourcePin::InputTime(pin_id.clone())))
            .or(self.inner.get_duration_back(pin_id, ctx))
    }

    fn get_time_fwd(&self, ctx: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.overrides
            .time
            .clone()
            .ok_or_else(|| GraphError::TimeUpdateMissingBack(TargetPin::OutputTime))
            .or(self.inner.get_time_fwd(ctx))
    }
}
