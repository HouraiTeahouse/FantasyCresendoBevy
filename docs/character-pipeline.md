# Fantasy Crescendo Character Development Guide

This is a public document which is a cursory guide that summarizes a number of
the process for creating and valildating the assets necessary to have for a
character that needs before it is considered ready to add to the game proper.

## Overview of the Process

The process includes several crucial steps that need to be done

 - Modeling
 - Texturing
 - Skinning
 - Rigging
 - Animating
 - Export from Blender
 - Frame data design
 - Packing for distribution
 - Validation.
 - Upload to build server.

## Modelling

TODO(james7132): Document

## Texturing

TODO(james7132): Document

### Adding color palletes.

A character is required to have at least **4** separate pallete swaps for
disambiguation. This generally only requires 4 separate albedo maps and 4
separate portraits for UIs.

Material design for characters should be done in the Blender Shading tab. Use the
Blender provided Principled BSDF shader and include at the minimum the character
albedo and normal maps. This must be done for all 4+ character palletes.

## Rigging

When rigging the character for animation, There are a few reseved bone names that
need to be kept in mind: (NOTE: these names are exact, and validation will
require inclusion):

 - `Ledge_Grab_Check` - An extra bone, usually attached as a child of the head.
   It's the focal point for where the game checks if there is a grabbable ledge
   while airborne. Only the global position is important.
 - `Ledge_Grab` - An extra bone, usually attached as a child to one of the hands,
   that signifies where the character is grabbing a ledge from. Only the global
   position is important.
 - `Player_Grab_Origin` - An extra bone, usually attached as a child to one of
   the hands, that signifies where the character is grabbing another player
   from. The global position and local rotation is important, as it will rotate
   the other player to match.
 - `Player_Grab_Target` - An extra bone, usually attached as a child of the upper
   chest. This is where the the character will be grabbed from when grabbed by an
   opponent. The global position and local rotation is important.

These reserved bones do not need to be animated: they generally can stay
statically bound to the parents and it's generally advised not to skin, but must
be included in the character's armature in Blender.

## Skinning

TODO(james7132): Document

## Animating

There are a number of reserved animation names that need to present when a
characer is loaded into the game. These are animations that involve specialized
and required behavior for all characters. Please see
`Appendix: Reserved Animations` for a full list.

If another character is already available for testing or public mocap data is
similar to the target animation, it may be advisable to use other humanoid
animations as a basis for the charater. NOTE: Some humanoid retargetting systems
require a specific bone structure and may not always be compatible with the
target rig.

TODO(james7132): Document availability and potential use of humanoid retargetting
software.

## Exporting Character from Blender

The Fantasy Crescendo game engine expects [GLTF](https://www.khronos.org/gltf/)
format as a rendering input.

Ensure the following checklist of things to have to ensure that the engine can
read the output files:

 - Export as \*.glb file. It's the most efficient encoding for the format.
 - Include: Untick everything.
 - Transform: Enable `+Y up`.
 - Geometry
   - Enable `UVs`, `Normals`, and `Vertex Colors`.
   - Set Materials to `Export`.
   - Disable compression. Blender offers mesh compression with Google Draco.
     The current engine does not support this in any way.
 - TIP: In the exporter, tick "Remember Export Settings" to ensure that the
   settigns are retained between exports.

## Importing into Character Editor
A custom character editor has been made for the game.

TODO(james7132): Make the character editor and add it here.

## Entering Frame Data
TODO(james7132): Make the character editor and add it here.

## Building Character
When everything is ready for use in a real game, the editor allows you to build a
compressed archive containing all of the necessary items to load the character in
game.

TODO(james7132): Make the character editor and add it here.

## Uploading to build server
The build system for Fantasy Crescendo keeps game assets separate from the code
during development and the final output for the game is set deliver the final
result only.

TODO(james7132): Make the character editor and add it here.

## Optional: Localization

## Appendix: Reserved Animations

These names are exact and must be available upon character validation. These do
not need to be distinct animations. If need be, copy animations from one to
another.

|Name|Description|
|:---|:----------|
|Action_Wait|Idle|
|Action_Walk|Walking|
|Action_Dash|Start run|
|Action_Run|Running|
|Action_RunBrake|Stop running|
|Action_Fall|Falling while airborne|
|Action_FallHelpless|Falling while disabled|
|Action_Grab|Grabbing another player|
|Action_Grabbed|Grabbed by another player|
|Action_LedgeGrab|Grabbing a ledge|
|Action_LedgeHang|Hanging onto a ledge|
|Action_LedgeClimb|Climbing up from a ledge|
|Action_LedgeTeeter|Teetering before falling off a ledge|
|Action_Turn|Turning while standing|
|Action_TurnDash|Turning while dashing|
|Action_TurnRun|Turning while running|
|Action_TurnRunBrake|Turning while stopping running|
|Action_JumpSquat|Prepare to jump|
|Action_JumpForward|Jump forwards|
|Action_JumpBackward|Jump backwards|
|Action_JumpAerialForwards|Jump while in air|
|Action_Squat|Crouching|
|Action_SquatEnter|Enter Crouching|
|Action_SquatEnter|Exiting Crouching|
|Action_LandHeavy|Hardlanding|
|Action_LandLight|Light landing|
|Action_LandAerial|Land during an aerial attack|
|Action_LandSpecial|Landing after falling helpless|
|Action_ShieldEnter|Start shielding|
|Action_Shield|Hold shield|
|Action_ShieldExit|Stop shielding|
|Action_ShieldStun|Broken shield|
|Action_Escape|Spot dodge|
|Action_EscapeForward|Roll forward|
|Action_EscapeBackward|Roll backward|
|Action_EscapeAir|Air dodge|
|Action_Rebound|Clang|
|Action_DamageFall|Tumble|

Other non-reserved, but recommended states (names don't need to be exact):
|Name|Description|
|:---|:----------|
|Action_Jab_{1, 2, 3}|Neutral Attack|
|Action_AttackDash|Dash Attack|
|Action_AttackUp|Up tilt|
|Action_AttackSide|Forward tilt|
|Action_AttackDown|Forward tilt|
|Action_AttackSmashUpStart|Up smash windup|
|Action_AttackSmashUpCharge|Charging Up smash|
|Action_AttackSmashUp|Up smash|
|Action_AttackSmashSideStart|Forward smash windup|
|Action_AttackSmashSideCharge|Charging forward smash|
|Action_AttackSmashSide|Forward smash|
|Action_AttackSmashSideStart|Down smash windup|
|Action_AttackSmashDownCharge|Charging down smash|
|Action_AttackSmashDown|Down smash|
|Action_AttackAir|Neutral air|
|Action_AttackAirUp|Up air|
|Action_AttackAirBack|Back air|
|Action_AttackAirForward|Forward air|
|Action_AttackAirDown|Forward air|
|Action_Special|Neutral special|
|Action_SpecialUp|Up special|
|Action_SpecialDown|Down special|
|Action_SpecialSide|Side special|
