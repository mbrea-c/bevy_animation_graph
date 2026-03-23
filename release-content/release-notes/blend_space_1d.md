---
title: Blend Space 1D Node
authors: ["@baszalmstra"]
pull_requests: []
---

A new `BlendSpace1DNode` has been added for single-axis parametric blending. This complements the existing 2D blend space node and is useful for common cases like blending between idle, walk, and run animations based on movement speed.

Configure the node with a list of named points along a single axis:

```ron
BlendSpace1DNode: (
    sync_mode: Absolute,
    points: [
        (id: "idle", value: 0.0),
        (id: "walk", value: 1.71),
        (id: "run",  value: 4.55),
    ],
)
```

At runtime, the node finds the two nearest points to the `parameter` input and linearly interpolates between them. All sync modes from the 2D blend space (Absolute, NoSync, EventTrack) are supported.
