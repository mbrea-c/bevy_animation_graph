"""
Blender script to convert Mixamo Locomotion Pack FBX files into a single GLB
with clean transforms (no armature rotation/scale, animation values in meters).

Usage:
    blender --background --python convert_mixamo.py -- <input_dir> <output.glb>

Example:
    blender --background --python convert_mixamo.py -- "path/to/Locomotion Pack" assets/models/mixamo_locomotion.glb

The script:
1. Imports the model FBX (X Bot.fbx) to get the mesh + armature
2. Applies armature rotation + scale so the exported GLB has identity transforms
3. Bakes all animations so keyframes are in the correct (meter, Y-up) space
4. Imports each animation FBX, bakes it, and transfers the action
5. Exports a single GLB with all animations
"""

import bpy
import sys
from pathlib import Path


def clean_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()
    for c in bpy.data.collections:
        bpy.data.collections.remove(c)
    for a in list(bpy.data.actions):
        bpy.data.actions.remove(a)


def get_args():
    argv = sys.argv
    if "--" not in argv:
        print("Usage: blender --background --python convert_mixamo.py -- <input_dir> <output.glb>")
        sys.exit(1)
    args = argv[argv.index("--") + 1:]
    if len(args) != 2:
        print("Usage: blender --background --python convert_mixamo.py -- <input_dir> <output.glb>")
        sys.exit(1)
    return Path(args[0]), Path(args[1])


def fbx_name_to_anim_name(fbx_path):
    return Path(fbx_path).stem.lower().replace(" ", "_")


def find_armature():
    for obj in bpy.data.objects:
        if obj.type == "ARMATURE":
            return obj
    return None


def scale_action_locations(action, scale):
    """Scale all location keyframe values in an action by the given factor.
    Blender 5.x uses layered actions: action -> layers -> strips -> channelbags -> fcurves."""
    for layer in action.layers:
        for strip in layer.strips:
            if strip.type != "KEYFRAME":
                continue
            for channelbag in strip.channelbags:
                for fcurve in channelbag.fcurves:
                    if fcurve.data_path.endswith(".location"):
                        for kf in fcurve.keyframe_points:
                            kf.co.y *= scale
                            kf.handle_left.y *= scale
                            kf.handle_right.y *= scale


def apply_transforms_and_bake(armature):
    """Apply armature rotation+scale and bake the current action so keyframes
    are in the new coordinate system."""

    # Remember the scale before applying (we need it to fix animation curves)
    orig_scale = armature.scale.x  # uniform scale

    # Select armature + children
    bpy.ops.object.select_all(action="DESELECT")
    armature.select_set(True)
    for child in armature.children:
        child.select_set(True)
    bpy.context.view_layer.objects.active = armature

    # Apply rotation + scale (fixes bone positions + mesh, but NOT animation curves)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    # Scale all animation location keyframes by the original scale factor
    # since transform_apply doesn't touch animation curves
    for action in bpy.data.actions:
        scale_action_locations(action, orig_scale)


def import_and_bake_animation(fbx_path, model_armature, anim_name):
    """Import an FBX, bake its animation in the correct space, transfer the
    action to the model armature, then clean up."""

    # Remember existing objects
    existing_objects = set(obj.name for obj in bpy.data.objects)
    existing_actions = set(a.name for a in bpy.data.actions)

    # Import the animation FBX
    bpy.ops.import_scene.fbx(filepath=str(fbx_path))

    # Find the newly imported armature
    new_armature = None
    for obj in bpy.data.objects:
        if obj.name not in existing_objects and obj.type == "ARMATURE":
            new_armature = obj
            break

    if new_armature is None:
        print(f"  WARNING: No armature found in {fbx_path}")
        return

    # Get the scale before applying transforms
    orig_scale = new_armature.scale.x

    # Apply transforms on the new armature
    bpy.ops.object.select_all(action="DESELECT")
    new_armature.select_set(True)
    bpy.context.view_layer.objects.active = new_armature
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    # Find the new action and scale its location keyframes
    new_action = None
    for a in bpy.data.actions:
        if a.name not in existing_actions:
            new_action = a
            break

    if new_action is None:
        print(f"  WARNING: No action found in {fbx_path}")
    else:
        scale_action_locations(new_action, orig_scale)
        new_action.name = anim_name
        new_action.use_fake_user = True
        print(f"  Action: {anim_name}")

    # Remove all newly imported objects
    for obj in list(bpy.data.objects):
        if obj.name not in existing_objects:
            bpy.data.objects.remove(obj, do_unlink=True)


def main():
    input_dir, output_path = get_args()

    fbx_files = sorted(input_dir.glob("*.fbx"), key=lambda p: p.stem.lower())
    model_file = None
    anim_files = []
    for f in fbx_files:
        if f.stem.lower() == "x bot":
            model_file = f
        else:
            anim_files.append(f)

    if model_file is None:
        print("ERROR: No 'X Bot.fbx' model file found.")
        sys.exit(1)

    print(f"Model: {model_file.name}")
    print(f"Animations ({len(anim_files)}):")
    for f in anim_files:
        print(f"  {f.name} -> {fbx_name_to_anim_name(f.name)}")

    clean_scene()

    # --- Import model ---
    print(f"\nImporting model: {model_file.name}")
    bpy.ops.import_scene.fbx(filepath=str(model_file))

    armature = find_armature()
    if armature is None:
        print("ERROR: No armature found")
        sys.exit(1)

    original_objects = set(obj.name for obj in bpy.data.objects)

    print(f"Armature: {armature.name}")
    print(f"  Pre-apply scale: {armature.scale[:]}")
    print(f"  Pre-apply rotation: {armature.rotation_euler[:]}")

    # Remove model's default action
    for action in list(bpy.data.actions):
        bpy.data.actions.remove(action)
    if armature.animation_data:
        armature.animation_data_clear()

    # Apply transforms on model armature
    apply_transforms_and_bake(armature)
    print(f"  Post-apply scale: {armature.scale[:]}")
    print(f"  Post-apply rotation: {armature.rotation_euler[:]}")

    # Print bone info
    for bone in armature.data.bones:
        if bone.parent is None:
            print(f"  Root bone: {bone.name}, head={bone.head_local[:]}")

    # --- Import animations ---
    for fbx_file in anim_files:
        anim_name = fbx_name_to_anim_name(fbx_file.name)
        print(f"Importing: {fbx_file.name} -> {anim_name}")
        import_and_bake_animation(fbx_file, armature, anim_name)

    # --- Push actions into NLA tracks ---
    if not armature.animation_data:
        armature.animation_data_create()
    for action in sorted(bpy.data.actions, key=lambda a: a.name):
        action.use_fake_user = True
        track = armature.animation_data.nla_tracks.new()
        track.name = action.name
        strip = track.strips.new(action.name, int(action.frame_range[0]), action)
        strip.name = action.name

    # --- Summary ---
    print(f"\nFinal objects:")
    for obj in bpy.data.objects:
        parent = obj.parent.name if obj.parent else "None"
        s = obj.scale
        print(f"  {obj.name}: type={obj.type}, parent={parent}, scale=({s.x:.4f}, {s.y:.4f}, {s.z:.4f})")

    print(f"\nAnimations ({len(bpy.data.actions)}):")
    for action in sorted(bpy.data.actions, key=lambda a: a.name):
        fr = action.frame_range
        duration = (fr[1] - fr[0]) / bpy.context.scene.render.fps
        print(f"  {action.name}: frames {fr[0]:.0f}-{fr[1]:.0f} ({duration:.2f}s)")

    # --- Export ---
    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        export_animations=True,
        export_nla_strips=True,
        use_active_scene=True,
    )

    print(f"\nExported to: {output_path}")
    print("Done!")


if __name__ == "__main__":
    main()
