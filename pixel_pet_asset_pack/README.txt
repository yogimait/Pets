PIXEL PET ASSET PACK
====================

Source: 27 characters extracted from the supplied reference image.
All individual character PNGs have transparent backgrounds.

Folders:
- characters/: character_01.png through character_27.png
- animations/character_XX/: GIF and horizontal PNG sprite sheets for:
  - idle
  - walk
  - jump
  - celebrate

Animation note:
These are lightweight procedural pixel animations made from the supplied single-pose sprites:
small bobbing, shifting, and bouncing motions. They do not contain newly drawn limb articulation.
For true walk cycles, waving, typing, sleeping, etc., each character would need additional hand-drawn or AI-generated poses.

Sprite dimensions: 112 x 160 px
Animation sheets: 6 frames horizontally, 672 x 160 px
GIFs: 6 frames, looping

Recommended use:
- Use transparent PNGs as desktop/project pets.
- Use PNG sprite sheets with a sprite animation system (e.g. CSS steps(), Canvas, Phaser, Godot).
- GIFs are convenient for quick previews.
