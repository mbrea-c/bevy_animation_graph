use std::borrow::Cow;

use bevy::{platform::collections::HashMap, reflect::Reflect};

use crate::{
    animation_graph::{GraphInputPin, SourcePin, TargetPin, TimeUpdate},
    context::new_context::GraphContext,
    duration_data::DurationData,
    edge_data::DataValue,
    errors::GraphError,
};

pub trait GraphIoEnv {
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DataValue, GraphError>;
    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
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
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DataValue, GraphError> {
        self.value.get_data_back(pin_id, ctx)
    }

    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
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
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        _: GraphContext,
    ) -> Result<DataValue, GraphError> {
        Err(GraphError::MissingGraphInputData(pin_id))
    }

    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
        _: GraphContext,
    ) -> Result<DurationData, GraphError> {
        Err(GraphError::MissingGraphInputDuration(pin_id))
    }

    fn get_time_fwd(&self, _: GraphContext) -> Result<TimeUpdate, GraphError> {
        Err(GraphError::TimeUpdateMissingBack(TargetPin::OutputTime))
    }
}
#[derive(Clone, Reflect, Default)]
pub struct IoOverrides {
    pub data: HashMap<GraphInputPin, DataValue>,
    pub duration: HashMap<GraphInputPin, DurationData>,
    pub time: Option<TimeUpdate>,
}

impl IoOverrides {
    pub fn clear(&mut self) {
        self.data.clear();
        self.duration.clear();
        self.time = None;
    }
}

impl GraphIoEnv for IoOverrides {
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        _: GraphContext,
    ) -> Result<DataValue, GraphError> {
        self.data
            .get(&pin_id)
            .cloned()
            .ok_or_else(|| GraphError::OutputMissing(SourcePin::InputData(pin_id.clone())))
    }

    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
        _: GraphContext,
    ) -> Result<DurationData, GraphError> {
        self.duration
            .get(&pin_id)
            .cloned()
            .ok_or_else(|| GraphError::DurationMissing(SourcePin::InputTime(pin_id.clone())))
    }

    fn get_time_fwd(&self, _: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.time
            .clone()
            .ok_or_else(|| GraphError::TimeUpdateMissingBack(TargetPin::OutputTime))
    }
}

/// Tries to get the value from T1 first, if that fails gets it from T2
#[derive(Clone)]
pub struct LayeredIoEnv<'a, T1: ToOwned, T2: ToOwned>(pub Cow<'a, T1>, pub Cow<'a, T2>);

impl<'a, T1: GraphIoEnv + Clone, T2: GraphIoEnv + Clone> GraphIoEnv for LayeredIoEnv<'a, T1, T2> {
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DataValue, GraphError> {
        self.0
            .get_data_back(pin_id.clone(), ctx.clone())
            .or_else(|_| self.1.get_data_back(pin_id, ctx))
    }

    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DurationData, GraphError> {
        self.0
            .get_duration_back(pin_id.clone(), ctx.clone())
            .or_else(|_| self.1.get_duration_back(pin_id, ctx))
    }

    fn get_time_fwd(&self, ctx: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.0
            .get_time_fwd(ctx.clone())
            .or_else(|_| self.1.get_time_fwd(ctx))
    }
}
