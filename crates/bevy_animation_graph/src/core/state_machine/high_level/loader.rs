use super::{serial::StateMachineSerial, State, StateMachine, Transition};
use crate::core::errors::AssetLoaderError;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    utils::BoxedFuture,
};

#[derive(Default)]
pub struct StateMachineLoader;

impl AssetLoader for StateMachineLoader {
    type Asset = StateMachine;
    type Settings = ();
    type Error = AssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await?;
            let serial: StateMachineSerial = ron::de::from_bytes(&bytes)?;
            let mut fsm = StateMachine::default();

            for state_serial in serial.states {
                fsm.add_state(State {
                    id: state_serial.id,
                    graph: load_context.load(state_serial.graph),
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
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fsm.ron"]
    }
}
