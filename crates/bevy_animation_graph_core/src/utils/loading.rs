use bevy::asset::LoadContext;

pub trait TryLoad<T> {
    type Error;

    fn try_load<'a, 'b>(&self, load_context: &'a mut LoadContext<'b>) -> Result<T, Self::Error>;
}
