---
title: Merge Event Queues Node
authors: ["@baszalmstra"]
pull_requests: [133]
---

A new `MergeEventQueues` node has been added that concatenates two event queues into one. This is useful when multiple event sources (e.g. a `FireEventNode` and a `ClipNode`'s event output) need to feed into a single event queue input like the FSM's driver.
