# A 3D model on a form: the `viewer3d` control (plan for a later project)

Written 2026-11-26 at the owner's word: "write a plan". Nothing here is built yet. It comes after
the `.phdb` database (docs/phdb-plan.md).

## What it is, and what it is not

A control dropped on a form like a Chart or a Picture. It opens a 3D model file (`.glb` / `.gltf`,
which Blender, SketchUp, FreeCAD and most 3D programs export) and lets the person turn it, zoom it
and click a part of it. The same form runs on Windows and as a web page.

Who it is for:
- a shop showing a product from every side
- a school: a heart, a molecule, the planets, a machine's parts
- a workshop: a part, with what it is made of or what it costs shown when it is clicked

It is **not** a game engine or a scene editor. Neither is planned. That is years of work in a
crowded field, and it is not what people using Studio need.

Today Phai has no 3D anywhere: phai-ui draws with GDI / GDI+ and phai-web with a 2D `<canvas>`.

## How

| Layer | What | Why |
|---|---|---|
| Drawing on the graphics card | [wgpu](https://github.com/gfx-rs/wgpu) | Pure Rust. One renderer for Windows (Direct3D 12 / Vulkan) and the web (WebGPU, or WebGL2 where WebGPU is missing). It takes a raw `HWND`, so it works under phai-ui's own windows without winit |
| Reading the model | [gltf](https://github.com/gltf-rs/gltf) | Pure Rust. Reads meshes, materials, textures and the parts' names |
| The maths | [glam](https://github.com/bitshifter/glam-rs) | Vectors, matrices and quaternions for the camera |
| Phai's own | `kind = "viewer3d"` in `.phf`; its properties in Studio's Properties pane; `Viewer3D` in the program's code; a guide page `ctl-viewer3d.html` in Thai | What people using Studio actually get |

The renderer stays small and fixed: physically based materials as glTF describes them, one sun
light plus an environment light, an optional shadow on the ground, and anti-aliasing. There are no
shaders for users to write and no scene graph to edit.

### Why wgpu and not Direct3D 11 directly

Direct3D 11 through the `windows` crate we already use would add nothing to the download. But the
web would then need a second renderer in WebGL2, and every fix would be made twice. wgpu gives one
renderer for both, at the cost of size and build time. So it goes behind a feature, as the
`browser` control does with WebView2: a program that has no `viewer3d` doesn't carry wgpu.

## The control

Properties (in `.phf` and the Properties pane):
- `Source`: the model file. It travels with the program as a Picture's image does
- `BackColor`
- `AutoRotate` (on / off, and its speed)
- `ShowGround` (a ground with a soft shadow)
- `CameraDistance`, `CameraAngle`: where it starts
- `AllowZoom`, `AllowTurn`

Events:
- `Loaded`
- `PartClick`: the part's name, as named in the model file (`wheel_front_left`)
- `PartHover`

Methods (with Thai names in the guide, as the Chart's are):
- `load(path)`
- `set_part_color(name, color)`
- `show_part(name, visible)`
- `select_part(name)`: outlines it
- `reset_camera()`
- `save_picture(path)`: a PNG, which also feeds `PHAI_SHOT` for tests

Mouse: drag to turn, wheel to zoom, right-drag to slide; one finger and two on a phone.

In Studio's designer the control shows the real model, as the Chart shows its real chart.

## Check first, when the project starts

1. **Licences.** Our runtime libraries are MIT, so theirs must allow that. We remember wgpu,
   gltf and glam as MIT/Apache-2.0, but we haven't checked, and wgpu pulls in naga and others.
2. **Size.** Measure a Release `score_book` with and without the feature, and the `.wasm` for the
   web. If wgpu adds too much, reconsider Direct3D 11 + WebGL2.
3. **Old computers.** Try it on a school-grade computer with integrated graphics and old drivers,
   and on one with no usable GPU (wgpu's fallback adapter / Windows' WARP). The control must say
   plainly that 3D can't be shown, never crash or show a black box.
4. **Under phai-ui.** A wgpu surface on a child `HWND` that phai-ui creates: resizing, DPI change,
   hiding on another tab, and many forms open at once.
5. **The web.** WebGPU in Chrome / Edge / Safari today, the WebGL2 fallback in the rest, and how a
   phone copes. Also the PWA's offline cache holding the `.glb`.

## A first prototype is done when

- A form with a `viewer3d` opens a `.glb` (with textures) on Windows; it turns, zooms and resets.
- `PartClick` gives the clicked part's name, and `set_part_color` colours it.
- The same program, published as a web page, does the same in Chrome and on a phone.
- Studio shows the model on the design canvas and lists the control in the Toolbox.
- On a computer without a usable GPU, the control shows a message and the rest of the form works.
- `save_picture` writes a PNG that the gallery's tests compare.

## Later, if people use it

- Playing the animations a glTF file carries (a door opening, an engine turning).
- Several models in one view, placed from code.
- Simple shapes from code (box, sphere, cylinder), for a teacher with no model file.
- Measuring the distance between two points clicked on the model.
