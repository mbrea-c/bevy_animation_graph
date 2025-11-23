use bevy::asset::{AssetLoader, LoadContext, io::Reader};

use super::{GlobalTransition, State, StateMachine, Transition, serial::StateMachineSerial};
use crate::errors::AssetLoaderError;

#[derive(Default)]
pub struct StateMachineLoader;

impl AssetLoader for StateMachineLoader {
    type Asset = StateMachine;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let serial: StateMachineSerial = ron::de::from_bytes(&bytes)?;
        let mut fsm = StateMachine {
            extra: serial.extra,
            node_spec: serial.node_spec,
            ..Default::default()
        };

        for state_serial in serial.states {
            let global_transition_data =
                state_serial.global_transition.map(|gt| GlobalTransition {
                    duration: gt.duration,
                    graph: load_context.load(gt.graph),
                });
            fsm.add_state(State {
                id: state_serial.id,
                graph: load_context.load(state_serial.graph),
                global_transition: global_transition_data,
            });
        }

        for transition_serial in serial.transitions {
            fsm.add_transition(Transition {
                id: transition_serial.id,
                source: transition_serial.source,
                target: transition_serial.target,
                duration: transition_serial.duration,
                graph: load_context.load(transition_serial.graph),
            });
        }

        fsm.set_start_state(serial.start_state);

        fsm.update_low_level_fsm();

        Ok(fsm)
    }

    fn extensions(&self) -> &[&str] {
        &["fsm.ron"]
    }
}
