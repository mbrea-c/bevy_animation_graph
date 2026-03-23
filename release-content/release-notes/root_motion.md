---
title: Root Motion Support
authors: ["@baszalmstra"]
pull_requests: [128]
---

This PR adds root motion extraction to the animation graph, following the approach discussed in #20 (Esoterica-style: embed root motion in the Pose, extract in ClipNode, let it propagate through the graph naturally).

The `ClipNode` can now extract per-frame displacement from the root bone and zero it in the visual pose. The delta is carried through the graph alongside the pose, so blends, transitions, loops, and mirroring all handle it automatically. At the end of the pipeline, an `extract_root_motion` system populates a `RootMotionOutput` component with the delta converted to entity-local space. The system does **not** apply the delta to the entity's Transform. That is left to user code, so you can integrate it with your own movement strategy (direct transform, physics, character controller, etc.).

Two extraction modes are supported:
- **Full**: extracts all translation + rotation, zeros the root bone completely
- **GroundPlane**: extracts only XZ translation + Y rotation, keeps vertical bob in the visual pose (the right choice for walking/running)

### Reference material

The design is based on Bobby Anguelov's Esoterica engine, specifically the approach described in #20 where root motion data is embedded in the pose result and propagated through the graph. I also looked at Esoterica's `RootMotionOverrideNode` and per-transition `RootMotionBlendMode` as inspiration for future work. Those aren't included in this PR to keep the scope manageable, but I plan to add them in a follow-up.

### Example

The `examples/root_motion` directory contains a full locomotion example with an FSM-driven animation graph (idle → walk → run → jump). The walk and run states use GroundPlane root motion, and the jump state auto-transitions back to idle when the clip finishes using a `MapEventsNode`.

The example uses animations from the Mixamo Locomotion Pack. Due to Mixamo's licensing terms, the `.glb` model file cannot be included in the repo. It is gitignored, and users need to download the pack themselves and run a Blender conversion script (`contrib/convert_mixamo.py`). The README has step-by-step instructions.

### Known limitations / future work

- **Transition root motion blending** is always linear. A per-transition `RootMotionBlendMode` (Blend / IgnoreSource / IgnoreTarget) would be useful for cases like walk-to-idle where the idle's zero root motion causes unnatural deceleration.
- **No root motion override node** yet. Esoterica has a `RootMotionOverrideNode` that lets gameplay code replace animation root motion with player-driven velocity on specific axes. This is essential for jump air control and could be added as a new graph node.
- **Extraction mode is per-ClipNode**, not per-state. A `RootMotionFilterNode` downstream would allow the same clip to use different modes in different contexts.
