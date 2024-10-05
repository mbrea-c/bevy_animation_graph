use std::any::{type_name, TypeId};

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

pub trait GetTypedExt {
    type Asset: Asset;

    fn get_typed(
        &self,
        handle: &Handle<Self::Asset>,
        untyped: &Assets<LoadedUntypedAsset>,
    ) -> Option<&Self::Asset>;
}

impl<A: Asset> GetTypedExt for Assets<A> {
    type Asset = A;

    fn get_typed(
        &self,
        handle: &Handle<Self::Asset>,
        untyped: &Assets<LoadedUntypedAsset>,
    ) -> Option<&Self::Asset> {
        let untyped_handle = handle.clone().untyped();
        let type_id = untyped_handle.type_id();
        if type_id == TypeId::of::<A>() {
            return self.get(handle);
        }

        let untyped_handle = untyped_handle
            .try_typed::<LoadedUntypedAsset>()
            .unwrap_or_else(|_| {
                panic!(
                    "if this handle isn't for `{}`, then it must be for `{}`",
                    type_name::<A>(),
                    type_name::<LoadedUntypedAsset>()
                )
            });
        let LoadedUntypedAsset { handle } = untyped.get(&untyped_handle)?;
        self.get(&handle.clone().typed::<A>())
    }
}
