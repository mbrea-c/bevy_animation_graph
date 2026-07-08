use bevy::{
    asset::{Assets, Handle},
    ecs::{
        entity::Entity,
        system::{In, InRef, Res, ResMut, StaticSystemParam, SystemParam, SystemParamItem},
        world::World,
    },
    platform::collections::HashMap,
};

use crate::{
    animation_clip::GraphClip,
    animation_graph::{AnimationGraph, GraphInputPin, PinId, SourcePin, TargetPin, TimeUpdate},
    animation_node::{AnimationNode, NodeLike},
    context::{
        deferred_gizmos::DeferredGizmos,
        graph_context::QueryOutputTime,
        graph_context_arena::GraphContextArena,
        io_env::GraphIoEnv,
        new_context::GraphContext,
        spec_context::{NodeInput, NodeOutput, NodeSpec, SpecContext, SpecResources},
        system_resources::SystemResources,
    },
    edge_data::DataValue,
    errors::GraphError,
    skeleton::Skeleton,
    state_machine::high_level::StateMachine,
};

pub struct GraphTestHarness<T> {
    env: TestIoEnv,
    graph_setup: T,
}

impl GraphTestHarness<NodeSpecSystemExtractor> {
    pub fn node(node_inner: impl NodeLike) -> Self {
        Self::new(NodeSpecSystemExtractor(AnimationNode::new(
            "test", node_inner,
        )))
    }
}

impl<T: SystemExtractor> GraphTestHarness<T> {
    pub fn new(setup: T) -> Self {
        GraphTestHarness {
            env: TestIoEnv::default(),
            graph_setup: setup,
        }
    }

    pub fn with_input_data(
        mut self,
        key: impl Into<GraphInputPin>,
        provider: impl Fn(TimeUpdate) -> DataValue + 'static,
    ) -> Self {
        self.env
            .data_handlers
            .insert(key.into(), RequestHandler::ByTime(Box::new(provider)));

        self
    }

    pub fn with_const_input_data(
        mut self,
        key: impl Into<GraphInputPin>,
        provider: impl Into<DataValue>,
    ) -> Self {
        self.env
            .data_handlers
            .insert(key.into(), RequestHandler::Const(provider.into()));

        self
    }

    pub fn with_input_duration(
        mut self,
        key: impl Into<GraphInputPin>,
        provider: impl Fn(TimeUpdate) -> Option<f32> + 'static,
    ) -> Self {
        self.env
            .duration_handlers
            .insert(key.into(), RequestHandler::ByTime(Box::new(provider)));

        self
    }

    pub fn with_const_input_duration(
        mut self,
        key: impl Into<GraphInputPin>,
        provider: impl Into<Option<f32>>,
    ) -> Self {
        self.env
            .duration_handlers
            .insert(key.into(), RequestHandler::Const(provider.into()));

        self
    }

    pub fn with_time_fwd(mut self, value: Option<TimeUpdate>) -> Self {
        self.env.time_fwd = value;
        self
    }

    pub fn when_queried(self) -> GraphTestResult {
        let mut world = World::new();
        let setup = self.setup(&mut world);

        let params = UnitTestSystemParams {
            graph: setup.graph,
            env: self.env,
        };

        let out = world
            .run_system_cached_with(Self::query_system, &params)
            .unwrap();

        GraphTestResult { result: out }
    }

    pub fn query_system(
        InRef(input): InRef<UnitTestSystemParams>,
        resources: SystemResources,
    ) -> Result<HashMap<PinId, DataValue>, GraphError> {
        let graph = resources.animation_graph_assets.get(&input.graph).unwrap();
        let mut arena = GraphContextArena::new(input.graph.id());
        let mut gizmos = DeferredGizmos::default();
        let global_input_data = HashMap::new();
        let entity_map = HashMap::new();

        let ctx = GraphContext::new(
            arena.get_toplevel_id(),
            graph,
            &mut arena,
            &resources,
            &input.env,
            Entity::PLACEHOLDER,
            &entity_map,
            &mut gizmos,
            &global_input_data,
        );
        graph.query_with_context(QueryOutputTime::None, ctx)
    }

    fn setup(&self, world: &mut World) -> UnitTestSetup {
        let graph = AnimationGraph::new();

        let mut assets = Assets::<AnimationGraph>::default();
        let handle = assets.add(graph);

        world.insert_resource(assets);

        world.insert_resource(Assets::<GraphClip>::default());
        world.insert_resource(Assets::<StateMachine>::default());
        world.insert_resource(Assets::<Skeleton>::default());

        let meta = world
            .run_system_cached_with(Self::extract, &self.graph_setup)
            .unwrap();
        world
            .run_system_cached_with(
                Self::apply_graph_setup,
                (&self.graph_setup, handle.clone(), meta),
            )
            .unwrap();

        UnitTestSetup { graph: handle }
    }

    fn extract(
        InRef(setup): InRef<T>,
        param: StaticSystemParam<T::Param<'static, 'static>>,
    ) -> T::Out {
        setup.process(param.into_inner())
    }

    fn apply_graph_setup(
        (InRef(setup), In(graph_handle), In(meta)): (
            InRef<T>,
            In<Handle<AnimationGraph>>,
            In<T::Out>,
        ),
        mut animation_graph_assets: ResMut<Assets<AnimationGraph>>,
    ) {
        let mut graph = animation_graph_assets.get_mut(&graph_handle).unwrap();
        setup.graph_setup(&mut graph, meta)
    }
}

#[derive(Debug)]
pub struct GraphTestResult {
    result: Result<HashMap<PinId, DataValue>, GraphError>,
}

impl GraphTestResult {
    pub fn then_output_is(
        &self,
        pin_id: impl Into<PinId>,
        expected: impl Into<DataValue>,
    ) -> &Self {
        let pin: PinId = pin_id.into();
        let val: DataValue = expected.into();
        assert_eq!(&val, self.result.as_ref().unwrap().get(&pin).unwrap());
        self
    }

    pub fn then_output_is_empty(&self) -> &Self {
        assert!(self.result.as_ref().unwrap().is_empty());
        self
    }
}

pub trait SystemExtractor: 'static {
    type Param<'w, 's>: SystemParam;
    type Out: 'static;

    fn process<'w, 's>(&self, param: SystemParamItem<Self::Param<'w, 's>>) -> Self::Out;
    fn graph_setup(&self, graph: &mut AnimationGraph, meta: Self::Out);
}

pub struct NodeSpecSystemExtractor(AnimationNode);

impl SystemExtractor for NodeSpecSystemExtractor {
    type Param<'w, 's> = (
        Res<'w, Assets<AnimationGraph>>,
        Res<'w, Assets<StateMachine>>,
    );
    type Out = NodeSpec;

    fn process<'w, 's>(
        &self,
        (graph_assets, fsm_assets): SystemParamItem<Self::Param<'w, 's>>,
    ) -> Self::Out {
        let res = SpecResources {
            graph_assets: &graph_assets,
            fsm_assets: &fsm_assets,
        };

        let mut node_spec = NodeSpec::default();

        let ctx = SpecContext::new(res, &mut node_spec);

        self.0.inner.spec(ctx).unwrap();

        node_spec
    }

    fn graph_setup(&self, graph: &mut AnimationGraph, meta: Self::Out) {
        graph.add_node(self.0.clone());

        for input in meta.sorted_inputs() {
            match input {
                NodeInput::Time(pin_id) => {
                    graph.add_input_time(GraphInputPin::Passthrough(pin_id.clone()));
                    graph.add_input_time_edge(
                        GraphInputPin::Passthrough(pin_id.clone()),
                        self.0.id,
                        pin_id,
                    );
                }
                NodeInput::Data(pin_id, data_spec) => {
                    graph.add_input_data(GraphInputPin::Passthrough(pin_id.clone()), data_spec);
                    graph.add_input_data_edge(
                        GraphInputPin::Passthrough(pin_id.clone()),
                        self.0.id,
                        pin_id,
                    );
                }
            }
        }

        for output in meta.sorted_outputs() {
            match output {
                NodeOutput::Time => {
                    graph.add_output_time();
                    graph.add_edge(SourcePin::NodeTime(self.0.id), TargetPin::OutputTime);
                }
                NodeOutput::Data(pin_id, data_spec) => {
                    graph.add_output_data(pin_id.clone(), data_spec);
                    graph.add_output_data_edge(self.0.id, pin_id.clone(), pin_id);
                }
            }
        }
    }
}

pub struct UnitTestSetup {
    pub graph: Handle<AnimationGraph>,
}

pub struct UnitTestSystemParams {
    graph: Handle<AnimationGraph>,
    env: TestIoEnv,
}

enum RequestHandler<T> {
    Const(T),
    ByTime(Box<dyn Fn(TimeUpdate) -> T + 'static>),
}

#[derive(Default)]
pub struct TestIoEnv {
    data_handlers: HashMap<GraphInputPin, RequestHandler<DataValue>>,
    duration_handlers: HashMap<GraphInputPin, RequestHandler<Option<f32>>>,
    time_fwd: Option<TimeUpdate>,
}

impl GraphIoEnv for TestIoEnv {
    fn get_data_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DataValue, GraphError> {
        match self
            .data_handlers
            .get(&pin_id)
            .ok_or(GraphError::MissingGraphInputData(pin_id.clone()))?
        {
            RequestHandler::Const(val) => Ok(val.clone()),
            RequestHandler::ByTime(f) => {
                let update = ctx.time_update_fwd(pin_id)?;
                Ok(f(update))
            }
        }
    }

    fn get_duration_back(
        &self,
        pin_id: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<crate::duration_data::DurationData, GraphError> {
        match self
            .duration_handlers
            .get(&pin_id)
            .ok_or(GraphError::MissingGraphInputData(pin_id.clone()))?
        {
            RequestHandler::Const(val) => Ok(*val),
            RequestHandler::ByTime(f) => {
                let update = ctx.time_update_fwd(pin_id)?;
                Ok(f(update))
            }
        }
    }

    fn get_time_fwd(&self, _: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.time_fwd.clone().ok_or(GraphError::TimeUpdateFailed)
    }
}
