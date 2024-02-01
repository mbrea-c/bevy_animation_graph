![Crates.io](https://img.shields.io/crates/v/bevy_animation_graph) ![Crates.io](https://img.shields.io/crates/d/bevy_animation_graph)
[![CI](https://github.com/mbrea-c/bevy_animation_graph/actions/workflows/ci.yaml/badge.svg)](https://github.com/mbrea-c/bevy_animation_graph/actions/workflows/ci.yaml)

# Bevy Animation Graph

## Motivation

Animation graphs are an essential tool for managing the complexity present in
the animation pipelines for modern 3D games. When your game has tens of
animations with complex blends and transitions, or you want to generate
your animations procedurally from very few keyframes, simple animation
clip playback is not enough.

This library aims to fill this gap in the Bevy ecosystem.

## Current Features

- Animation graphs are assets. They can be loaded from asset files, or created in code with an ergonomic API.
- Visual graph editor.
- Available nodes:
  - Animation chaining (i.e. play one node after another).
  - Two-bone inverse kinematics.
  - Looping.
  - Linear Blending (in bone space).
  - Mirror animation about the YZ plane.
  - Animation clip playback.
  - Apply a given rotation to some bones in a pose using a bone mask.
  - Arithmetic nodes:
    - F32: Add, Subtract, Multiply, Divide, Clamp.
    - Vec3: Rotation arc.
  - Speed up or slow down animation playback.
  - Animation graph node.
- Nesting animation graphs as nodes within other graphs.
- Export animation graphs in graphviz `.dot` format for visualization.
- Output from graph nodes is cached to avoid unnecessary computations.
- Support for custom nodes written in Rust (with the caveat that custom nodes cannot be serialized/deserialized as assets).

Feel free to request new nodes if some feature you need is currently missing.

## Planned Features

Being worked on:

1. Finite state machines.
1. Synchronization tracks.

Wishlist:

1. Ragdoll and physics integration (inititally `bevy_xpbd`, possibly rapier later):
   1. Using a bone mask to specify which bones are kinematically driven, and which bones are simulated (i.e. _ragdolled_)
   2. Pose matching with joint motors (pending on joint motors being implemented in `bevy_xpbd`, currently WIP)
1. FABRIK node (?).

## Installation

This project is divided in two crates:

- [bevy_animation_graph](https://crates.io/crates/bevy_animation_graph) is the
  library part of this project. This should be added as a dependency to your
  project in order to use animation graphs. To install the latest published version from crates.io run

  ```bash
  cargo add bevy_animation_graph
  ```

  or manually add the latest version to your `Cargo.toml`.

  To install the latest git master, add the following to `Cargo.toml`

  ```toml
  # ...
  [dependencies]
  # ...
  bevy_animation_graph = { git = "https://github.com/mbrea-c/bevy_animation_graph.git" }
  # ...
  ```

- [bevy_animation_graph_editor](https://crates.io/crates/bevy_animation_graph_editor)
  is the editor. You can install like you would install any other rust binary:

  ```bash
  # for the latest crates.io version
  cargo install bevy_animation_graph_editor
  # for the latest master
  cargo install --git https://github.com/mbrea-c/bevy_animation_graph bevy_animation_graph_editor
  # for the version from a local workspace
  cargo install --path <PATH_TO_WORKSPACE> bevy_animation_graph_editor

  # use the --force flag to force reinstall
  ```

## Usage and examples

The documentation in [docs.rs](https://docs.rs/bevy_animation_graph) contains an
introduction of the library and editor and an explanation of a simple animation graph example.
See also the video below for a demonstration of editor usage.

The complete example is also included in the
[examples/fox/](examples/fox/examples/fox.rs) package.

A more complex example is included in the [examples/human_ik/](examples/human_ik/examples/human_ik.rs) package.

### Screenshots

![Locomotion graph example](locomotion_graph.png)

### Editor usage demonstration video

In YouTube:

[![Demo](https://img.youtube.com/vi/q-JBSQJIcX0/hqdefault.jpg)](https://www.youtube.com/watch?v=q-JBSQJIcX0)

## Contributing

If you run into a bug or want to discuss potential new features, feel free to post an issue, open a PR or reach out to me in Discord
(@mbreac in the Bevy discord).

## FAQ

### Is this ready for production?

Depends.

It can already be useful for small-ish projects, but I cannot guarantee
API stability between different `0.x` versions (it is a big library, it is
relatively young and I don't have previous experience with animation programming,
so I'm still figuring out the best ways of doing things).
This means that it will likely be necessary to go into your animation graph
assets and manually migrate them between versions, at least until I find a
better way to handle migrations.

Additionally, there ~~may~~ will be bugs and other issues. I try to get them
fixed as they come up.

### Will you implement feature X?

If it's a small feature (e.g. some additional vector or floating point
arithmetic node) it's likely that I have just not got around to it. If you
open an issue I will probably implement it quickly. PRs are also welcome.

For larger features, it's better to start by opening an issue for discussion or
pinging me in the Bevy discord.

## Acknowledgements

Many thanks to [Bobby Anguelov](https://www.youtube.com/@BobbyAnguelov) for his lectures on animation programming.
