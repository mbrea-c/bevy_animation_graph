---
title: Boolean Logic Nodes
authors: ["@baszalmstra"]
pull_requests: [132]
---

Three new boolean logic nodes have been added to complement the existing `ConstBool` node:

- **`NotBool`** (`! Not`): Logical NOT — inverts a boolean input
- **`AndBool`** (`&& And`): Logical AND — true only when both inputs are true
- **`OrBool`** (`|| Or`): Logical OR — true when either input is true

These enable combining boolean conditions within the graph — e.g. `is_grounded AND is_moving` to drive blend parameters or FSM transitions.
