use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
};

use super::{StateMachine, serial::StateMachineSerial};
use crate::{errors::AssetLoaderError, utils::loading::TryLoad};

#[derive(Default, TypePath)]
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
            editor_metadata: serial.editor_metadata,
            node_spec: serial.node_spec,
            ..Default::default()
        };

        for state_serial in serial.states {
            fsm.add_state(state_serial.try_load(load_context)?);
        }

        for transition_serial in serial.transitions {
            fsm.add_transition_unchecked(transition_serial.try_load(load_context)?);
        }

        fsm.set_start_state(serial.start_state);

        fsm.update_low_level_fsm();

        Ok(fsm)
    }

    fn extensions(&self) -> &[&str] {
        &["fsm.ron"]
    }
}
