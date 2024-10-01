use crate::egui_fsm::{
    link::{LinkStyleArgs, TransitionSpec},
    node::StateSpec,
};
use bevy::{asset::Assets, math::Vec2, utils::HashMap};
use bevy_animation_graph::core::state_machine::{
    high_level::{State, StateMachine, Transition},
    FSMState, StateId, TransitionId,
};
use bevy_inspector_egui::egui::Color32;

pub struct StateIndices {
    name_to_idx: HashMap<String, usize>,
    idx_to_name: HashMap<usize, String>,
    count: usize,
}

impl Default for StateIndices {
    fn default() -> Self {
        Self {
            name_to_idx: HashMap::default(),
            idx_to_name: HashMap::default(),
            count: 4, // 0, 1, 2, 3 are reserved for input/output nodes
        }
    }
}

impl StateIndices {
    pub fn add_mapping(&mut self, name: String) -> usize {
        let id = self.count;
        self.count += 1;

        self.name_to_idx.insert(name.clone(), id);
        self.idx_to_name.insert(id, name);

        id
    }

    pub fn get_id(&self, name: &str) -> Option<usize> {
        self.name_to_idx.get(name).copied()
    }

    pub fn name(&self, id: usize) -> Option<&String> {
        self.idx_to_name.get(&id)
    }
}

#[derive(Default)]
pub struct TransitionIndices {
    name_to_idx: HashMap<(usize, TransitionId, usize), usize>,
    idx_to_name: HashMap<usize, (usize, TransitionId, usize)>,
    count: usize,
}

impl TransitionIndices {
    pub fn add_mapping(&mut self, start_id: usize, transition_id: TransitionId, end_id: usize) {
        let id = self.count;
        self.count += 1;

        self.name_to_idx
            .insert((start_id, transition_id.clone(), end_id), id);
        self.idx_to_name
            .insert(id, (start_id, transition_id, end_id));
    }

    pub fn id(&self, start_id: usize, transition_id: TransitionId, end_id: usize) -> Option<usize> {
        self.name_to_idx
            .get(&(start_id, transition_id, end_id))
            .copied()
    }

    pub fn edge(&self, id: usize) -> Option<&(usize, TransitionId, usize)> {
        self.idx_to_name.get(&id)
    }
}

#[derive(Default)]
pub struct FsmIndices {
    pub state_indices: StateIndices,
    pub transition_indices: TransitionIndices,
}

impl FsmIndices {
    pub fn transition_ids(
        &self,
        source_state: StateId,
        transition: TransitionId,
        target_state: StateId,
    ) -> Option<(usize, usize, usize)> {
        let source_id = self.state_indices.get_id(&source_state)?;
        let target_id = self.state_indices.get_id(&target_state)?;

        let transition_id = self
            .transition_indices
            .id(source_id, transition, target_id)?;

        Some((transition_id, source_id, target_id))
    }
}

pub fn make_fsm_indices(
    graph: &StateMachine,
    _ctx: &Assets<StateMachine>,
) -> Result<FsmIndices, Vec<TransitionId>> {
    let mut fsm_indices = FsmIndices::default();

    for state in graph.states.values() {
        // add node
        fsm_indices.state_indices.add_mapping(state.id.clone());
    }

    let mut remove_edges = vec![];

    // Add transtions
    for transition in graph.transitions.values() {
        let Some(source_id) = fsm_indices.state_indices.get_id(&transition.source) else {
            remove_edges.push(transition.id.clone());
            continue;
        };
        let Some(target_id) = fsm_indices.state_indices.get_id(&transition.target) else {
            remove_edges.push(transition.id.clone());
            continue;
        };
        fsm_indices
            .transition_indices
            .add_mapping(source_id, transition.id.clone(), target_id);
    }

    if remove_edges.is_empty() {
        Ok(fsm_indices)
    } else {
        Err(remove_edges)
    }
}

#[derive(Default)]
pub struct FsmReprSpec {
    pub states: Vec<StateSpec>,
    pub transitions: Vec<TransitionSpec>,
}

impl FsmReprSpec {
    pub fn from_fsm(
        fsm: &StateMachine,
        indices: &FsmIndices,
        fsm_assets: &Assets<StateMachine>,
        fsm_state: Option<FSMState>,
    ) -> Self {
        let mut repr_spec = FsmReprSpec::default();

        repr_spec.add_states(fsm, indices, fsm_assets, fsm_state.clone());
        repr_spec.add_transitions(fsm, indices, fsm_assets, fsm_state);

        repr_spec
    }

    fn node_debug_info(node: &State, fsm_state: &Option<FSMState>) -> bool {
        let Some(fsm_state) = fsm_state else {
            return false;
        };

        fsm_state.state == node.id
    }

    fn transition_debug_info(
        transition: &Transition,
        fsm_state: &Option<FSMState>,
        fsm: &StateMachine,
    ) -> bool {
        let Some(fsm_state) = fsm_state else {
            return false;
        };

        fsm.get_low_level_fsm()
            .states
            .get(&fsm_state.state)
            .and_then(|s| s.transition.as_ref())
            .map(|t| t.hl_transition_id == transition.id)
            .unwrap_or(false)
    }

    fn add_states(
        &mut self,
        fsm: &StateMachine,
        indices: &FsmIndices,
        _fsm_assets: &Assets<StateMachine>,
        fsm_state: Option<FSMState>,
    ) {
        for state in fsm.states.values() {
            let active = Self::node_debug_info(state, &fsm_state);

            let constructor = StateSpec {
                id: indices.state_indices.get_id(&state.id).unwrap(),
                name: state.id.clone(),
                subtitle: "".into(),
                origin: fsm
                    .extra
                    .states
                    .get(&state.id)
                    .copied()
                    .unwrap_or(Vec2::default())
                    .to_array()
                    .into(),
                time: None,
                duration: None,
                active,
                is_start_state: state.id == fsm.start_state,
                has_global_transition: state.global_transition.is_some(),
                ..Default::default()
            };

            self.states.push(constructor);
        }
    }

    fn add_transitions(
        &mut self,
        fsm: &StateMachine,
        indices: &FsmIndices,
        _fsm_assets: &Assets<StateMachine>,
        fsm_state: Option<FSMState>,
    ) {
        for transition in fsm.transitions.values() {
            let active = Self::transition_debug_info(transition, &fsm_state, fsm);

            let (edge_id, source_id, target_id) = indices
                .transition_ids(
                    transition.source.clone(),
                    transition.id.clone(),
                    transition.target.clone(),
                )
                .unwrap();

            let base = Some(Color32::from_rgba_unmultiplied(150, 150, 150, 127));
            let hovered = Some(Color32::from_rgb(200, 200, 200));
            let selected = Some(Color32::from_rgb(200, 200, 200));

            self.transitions.push(TransitionSpec {
                id: edge_id,
                start_pin_index: source_id,
                end_pin_index: target_id,
                style: LinkStyleArgs {
                    base,
                    hovered,
                    selected,
                    thickness: Some(5.),
                },
                active,
            });
        }
    }
}
