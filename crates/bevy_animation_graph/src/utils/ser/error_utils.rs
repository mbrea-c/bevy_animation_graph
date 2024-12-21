use core::fmt::Display;
use serde::ser::Error;

/// A helper function for generating a custom serialization error message.
///
/// This function should be preferred over [`Error::custom`] as it will include
/// other useful information, such as the [type info stack].
///
/// [type info stack]: crate::type_info_stack::TypeInfoStack
pub(super) fn make_custom_error<E: Error>(msg: impl Display) -> E {
    E::custom(msg)
}
