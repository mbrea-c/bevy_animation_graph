## Motivation

Animation graphs are an essential tool for managing the complexity present in
the animation pipelines for modern 3D games. When your game has tens of
animations with complex blends and transitions, or you want to generate
your animations procedurally from very few keyframes, simple animation
clip playback is not enough.

This library aims to fill this gap in the Bevy ecosystem.

## Current Features

- Animation graphs are assets. They can be loaded from asset files, or created in code with an ergonomic API.
- Available nodes:
  - Animation clip playback
  - Animation chaining (i.e. play one node after another)
  - Looping
  - Linear Blending (in bone space)
  - Mirror animation about the YZ plane
  - Arithmetic nodes:
    - F32: Add, Subtract, Multiply, Divide, Clamp.
  - Speed up or slow down animation playback
- Support for custom nodes written in Rust (with the caveat that custom nodes cannot be serialized/deserialized as assets)

## Planned Features

In order of priority:
1. Finite state machines.
1. More procedural animation nodes:
    1. Apply transform to bone
    2. Two-bone IK
2. Ragdoll and physics integration (inititally `bevy_xpbd`):
    1. Using a bone mask to specify which bones are kinematically driven, and which bones are simulated (i.e. *ragdolled*)
    2. Pose matching with joint motors (pending on joint motors being implemented in `bevy_xpbd`, currently WIP)
3. FABRIK node.

## Example
