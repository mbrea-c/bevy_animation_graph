use std::any::type_name;

use bevy::{asset::LoadedUntypedAsset, prelude::*};

// When we load in a node from the graph loader, its handles won't
// actually be `Handle<T>`s, but will be `Handle<LoadedUntypedAsset>`.
//
// We can't fix this right now because of the way the asset server works right
// now, where we have to do some IO before figuring out the type of a handle.
//
// So what we do here is, if we have one of these untyped handles, we
// look up what handle it actually points to, and use that one for lookups
// instead.
//
// TODO: also potentially replace the original handle with the properly typed
// one

pub fn look_up<'t, A: Asset>(
    handle: &Handle<A>,
    untyped: &Assets<LoadedUntypedAsset>,
    typed: &'t Assets<A>,
) -> Option<&'t A> {
    if let Some(asset) = typed.get(handle) {
        return Some(asset);
    }

    let untyped_handle = handle
        .clone()
        .untyped()
        .try_typed::<LoadedUntypedAsset>()
        .unwrap_or_else(|_| {
            panic!(
                "{handle:?} must either point to a `{}` or `{}`",
                type_name::<A>(),
                type_name::<LoadedUntypedAsset>(),
            )
        });
    let LoadedUntypedAsset { handle } = untyped.get(&untyped_handle)?;
    typed.get(&handle.clone().typed::<A>())
}
