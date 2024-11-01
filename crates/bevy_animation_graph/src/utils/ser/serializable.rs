use core::ops::Deref;

use bevy::reflect::erased_serde;

/// A type-erased serializable value.
pub enum Serializable<'a> {
    Owned(Box<dyn erased_serde::Serialize + 'a>),
    Borrowed(&'a dyn erased_serde::Serialize),
}

impl<'a> Deref for Serializable<'a> {
    type Target = dyn erased_serde::Serialize + 'a;

    fn deref(&self) -> &Self::Target {
        match self {
            Serializable::Borrowed(serialize) => serialize,
            Serializable::Owned(serialize) => serialize,
        }
    }
}
