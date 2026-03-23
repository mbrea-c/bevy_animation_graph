# Root Motion Locomotion Example

Demonstrates root motion extraction and application using the Mixamo Locomotion Pack.
The character uses an FSM-driven animation graph with idle, walk, run, and jump states.
Walking and running animations extract root motion via `GroundPlane` mode, which drives
the character's forward movement from the animation data itself.

The plugin extracts the root motion delta into a `RootMotionOutput` component but does
**not** apply it automatically. The example's own `apply_root_motion` system reads the
delta and moves the entity, showing how to integrate root motion with your own movement
logic.

## Setup

This example requires a manual setup step because the Mixamo animation assets cannot be
redistributed. Mixamo's [terms of use](https://www.adobe.com/legal/terms.html) allow
using their assets in projects but not distributing the raw files in a public repository.
The `.glb` model file is therefore `.gitignore`d, and you need to download and convert
the animations yourself.

### 1. Download the Locomotion Pack from Mixamo

1. Go to [mixamo.com](https://www.mixamo.com/) and sign in (free Adobe account required)
2. Search for **"Locomotion Pack"** in the animations tab
3. Download the pack (FBX format, with skin)
4. Extract the ZIP to a temporary directory

### 2. Convert FBX to GLB

The FBX files need to be merged into a single `.glb` file using Blender (4.0+):

```bash
blender --background --python contrib/convert_mixamo.py -- \
    "path/to/Locomotion Pack" \
    examples/root_motion/assets/models/mixamo_locomotion.glb
```

This imports the mesh, applies armature transforms, scales animation keyframes
to meters, and exports all animations into one GLB.

### 3. Run the example

```bash
cargo run -p root_motion --example root_motion
```

## Controls

| Key | Action |
|-----|--------|
| **W** | Walk forward |
| **Shift+W** | Run |
| **A / D** | Turn left / right |
| **Space** | Jump |
| **R** | Reset position and animation |

## How it works

The animation graph uses an FSM (`fsm/locomotion_rm.fsm.ron`) with four states:

- **Idle**: Looping idle animation, no root motion
- **Walk**: Looping walk with `GroundPlane` root motion extraction
- **Run**: Looping run with `GroundPlane` root motion extraction
- **Jump**: One-shot jump animation, no root motion. Automatically transitions back
  to idle when the clip finishes (via `MapEventsNode` converting `AnimationClipFinished`
  into a `TransitionToStateLabel("idle")` event)

Transitions between states use a blend graph (`blend_transition.animgraph.ron`) with
configurable durations. The example code sends `TransitionToStateLabel` events each
frame based on keyboard input.

The `RootMotionOutput` component is populated by the plugin's `extract_root_motion`
system. The example's `apply_root_motion` system then reads the delta and applies it:

```rust
fn apply_root_motion(mut query: Query<(&RootMotionOutput, &mut Transform)>) {
    for (rm, mut transform) in &mut query {
        let world_delta = transform.rotation * rm.translation_delta;
        transform.translation += world_delta;
        transform.rotation *= rm.rotation_delta;
    }
}
```

This separation lets you replace direct transform manipulation with physics impulses,
character controller movement, or any other strategy.
