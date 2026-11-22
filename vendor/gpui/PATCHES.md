# Astreo GPUI patch

Source: crates.io `gpui` 0.2.2, copyright Zed Industries, licensed under Apache-2.0.
The original `LICENSE-APACHE` and other upstream notices are preserved.

Modified files:

- `build.rs`
- `src/scene.rs`
- `src/window.rs`
- `src/style.rs`
- `src/styled.rs`
- `src/platform/mac/metal_renderer.rs`
- `src/platform/mac/shaders.metal`
- `src/platform/blade/blade_renderer.rs`
- `src/platform/windows/directx_renderer.rs`

Astreo adds an in-window backdrop filter to the Metal renderer. Paired scene
operations capture the border box intersected with the content mask and viewport,
rounded out to device pixels. MPSImageGaussianBlur mirrors including the edge
pixel on macOS 13+ (MPSImageEdgeModeMirrorWithEdge); older systems use clamp.
Exact scratch sizes keep the filtering boundary at the captured edge. Shadows
paint inside a layer covering the union of the element and every shadow's spread
and three blur radii, enforcing capture → shadows → composite for any offset.
The composite compensates for the solid tint's alpha so the filtered backdrop and
tint fade as one CSS group, while rounded SDF coverage hides the element's own
shadow. Logical blur radii scale with the display. Scratch textures and blur
kernels are reused and released when glass disappears; the other renderers ignore
backdrop operations.
The Metal layer enables framebufferOnly for frames without glass and disables it
before requesting a drawable for glass frames. A drawable vended before the switch
may still be framebuffer-only; capture skips it and leaves the tint for that frame.
Scratch textures retain only the current frame's captures, so resizing glass does
not grow the pool. If MPS is unavailable, glass composites the unblurred snapshot
before its tint and logs once.
The patch is limited to the files above to simplify rebasing.

Apple API references:

- https://developer.apple.com/documentation/metalperformanceshaders/mpsimagegaussianblur
- https://developer.apple.com/documentation/metalperformanceshaders/mpsunaryimagekernel/encode(commandbuffer:sourcetexture:destinationtexture:)
