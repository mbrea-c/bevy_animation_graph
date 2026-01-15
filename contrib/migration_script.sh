#!/bin/bash

# Default values
CUSTOM_DIR=""

# Parse command-line options
while getopts ":c:" opt; do
    case "${opt}" in
    c) CUSTOM_DIR="${OPTARG}" ;; # -c <path>
    *)
        echo "Invalid option: -${opt}" >&2
        exit 1
        ;;
    esac
done
shift $((OPTIND - 1))

if [ -z "$CUSTOM_DIR" ]; then
    MIGRATION_DIR_RELPATH="./migration-tmp-dir-$(uuidgen)"
else
    MIGRATION_DIR_RELPATH="$CUSTOM_DIR"
fi
CURRENT_ASSETS_RELPATH="./assets"

MIGRATION_DIR="$(realpath "$MIGRATION_DIR_RELPATH")"
CURRENT_ASSETS_DIR="$(realpath "$CURRENT_ASSETS_RELPATH")"

if [ -z "$CUSTOM_DIR" ]; then
    mkdir -p "$MIGRATION_DIR"
fi

OLD_ASSETS_DIR="$(realpath "$MIGRATION_DIR/assets.bak")"

echo "$CURRENT_ASSETS_DIR"
echo "$OLD_ASSETS_DIR"
echo "$MIGRATION_DIR"

if [ -z "$CUSTOM_DIR" ]; then
    cp -r "$CURRENT_ASSETS_DIR" "$OLD_ASSETS_DIR"
fi

cd "$MIGRATION_DIR" || exit

if [ -z "$CUSTOM_DIR" ]; then
    (git clone 'https://github.com/mbrea-c/bevy_animation_graph.git' && mv bevy_animation_graph bevy_animation_graph_old && cd bevy_animation_graph_old && git checkout v0.8.0)
    (git clone 'https://github.com/mbrea-c/bevy_animation_graph.git' && mv bevy_animation_graph bevy_animation_graph_new && cd bevy_animation_graph_new && git checkout refactor_and_fsm_overhaul)

    (cd bevy_animation_graph_old && cargo build --release)
    (cd bevy_animation_graph_new && cargo build --release)
fi

(cd bevy_animation_graph_old && cargo run --release -p bevy_animation_graph_editor -- -a "$OLD_ASSETS_DIR") &
(cd bevy_animation_graph_new && cargo run --release -p bevy_animation_graph_editor -- -a "$CURRENT_ASSETS_DIR") &

wait
