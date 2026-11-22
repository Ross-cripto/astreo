// Modified for Astreo: backdrop blur (see PATCHES.md).
// todo("windows"): remove
#![cfg_attr(windows, allow(dead_code))]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AtlasTextureId, AtlasTile, BackdropBlur, Background, Bounds, ContentMask, Corners, Edges, Hsla,
    Pixels, Point, Radians, ScaledPixels, Size, bounds_tree::BoundsTree, point, px,
};
use std::{
    fmt::Debug,
    iter::Peekable,
    ops::{Add, Range, Sub},
    slice,
};

#[allow(non_camel_case_types, unused)]
pub(crate) type PathVertex_ScaledPixels = PathVertex<ScaledPixels>;

pub(crate) type DrawOrder = u32;

#[derive(Default)]
pub(crate) struct Scene {
    pub(crate) paint_operations: Vec<PaintOperation>,
    primitive_bounds: BoundsTree<ScaledPixels>,
    layer_stack: Vec<DrawOrder>,
    pub(crate) backdrops: Vec<Backdrop>,
    next_backdrop_id: u32,
    backdrop_stack: Vec<u32>,
    pub(crate) shadows: Vec<Shadow>,
    pub(crate) quads: Vec<Quad>,
    pub(crate) paths: Vec<Path<ScaledPixels>>,
    pub(crate) underlines: Vec<Underline>,
    pub(crate) monochrome_sprites: Vec<MonochromeSprite>,
    pub(crate) polychrome_sprites: Vec<PolychromeSprite>,
    pub(crate) surfaces: Vec<PaintSurface>,
}

impl Scene {
    pub fn clear(&mut self) {
        self.paint_operations.clear();
        self.primitive_bounds.clear();
        self.layer_stack.clear();
        self.backdrops.clear();
        self.next_backdrop_id = 0;
        self.backdrop_stack.clear();
        self.paths.clear();
        self.shadows.clear();
        self.quads.clear();
        self.underlines.clear();
        self.monochrome_sprites.clear();
        self.polychrome_sprites.clear();
        self.surfaces.clear();
    }

    pub fn len(&self) -> usize {
        self.paint_operations.len()
    }

    pub fn push_layer(&mut self, bounds: Bounds<ScaledPixels>) {
        let order = self.primitive_bounds.insert(bounds);
        self.layer_stack.push(order);
        self.paint_operations
            .push(PaintOperation::StartLayer(bounds));
    }

    pub fn pop_layer(&mut self) {
        self.layer_stack.pop();
        self.paint_operations.push(PaintOperation::EndLayer);
    }

    pub fn insert_primitive(&mut self, primitive: impl Into<Primitive>) {
        let mut primitive = primitive.into();
        if let Primitive::Backdrop(backdrop) = &mut primitive {
            debug_assert!(
                self.layer_stack.is_empty(),
                "glass inside paint_layer is unsupported"
            );
            match backdrop.phase {
                BackdropPhase::Capture => {
                    backdrop.id = self.next_backdrop_id;
                    self.next_backdrop_id += 1;
                    self.backdrop_stack.push(backdrop.id);
                }
                BackdropPhase::Composite => {
                    let capture_id = self.backdrop_stack.pop();
                    debug_assert!(capture_id.is_some(), "backdrop composite has no capture");
                    backdrop.id = capture_id.unwrap_or(u32::MAX);
                }
            }
            self.paint_operations
                .push(PaintOperation::Primitive(primitive.clone()));
        }
        let clipped_bounds = primitive
            .bounds()
            .intersect(&primitive.content_mask().bounds);

        if clipped_bounds.is_empty() {
            return;
        }

        let order = self
            .layer_stack
            .last()
            .copied()
            .unwrap_or_else(|| self.primitive_bounds.insert(clipped_bounds));
        match &mut primitive {
            Primitive::Backdrop(backdrop) => {
                backdrop.order = order;
                self.backdrops.push(backdrop.clone());
            }
            Primitive::Shadow(shadow) => {
                shadow.order = order;
                self.shadows.push(shadow.clone());
            }
            Primitive::Quad(quad) => {
                quad.order = order;
                self.quads.push(quad.clone());
            }
            Primitive::Path(path) => {
                path.order = order;
                path.id = PathId(self.paths.len());
                self.paths.push(path.clone());
            }
            Primitive::Underline(underline) => {
                underline.order = order;
                self.underlines.push(underline.clone());
            }
            Primitive::MonochromeSprite(sprite) => {
                sprite.order = order;
                self.monochrome_sprites.push(sprite.clone());
            }
            Primitive::PolychromeSprite(sprite) => {
                sprite.order = order;
                self.polychrome_sprites.push(sprite.clone());
            }
            Primitive::Surface(surface) => {
                surface.order = order;
                self.surfaces.push(surface.clone());
            }
        }
        if !matches!(primitive, Primitive::Backdrop(_)) {
            self.paint_operations
                .push(PaintOperation::Primitive(primitive));
        }
    }

    pub fn replay(&mut self, range: Range<usize>, prev_scene: &Scene) {
        for operation in &prev_scene.paint_operations[range] {
            match operation {
                PaintOperation::Primitive(primitive) => self.insert_primitive(primitive.clone()),
                PaintOperation::StartLayer(bounds) => self.push_layer(*bounds),
                PaintOperation::EndLayer => self.pop_layer(),
            }
        }
    }

    pub fn finish(&mut self) {
        self.backdrops.sort_by_key(|backdrop| backdrop.order);
        self.shadows.sort_by_key(|shadow| shadow.order);
        self.quads.sort_by_key(|quad| quad.order);
        self.paths.sort_by_key(|path| path.order);
        self.underlines.sort_by_key(|underline| underline.order);
        self.monochrome_sprites
            .sort_by_key(|sprite| (sprite.order, sprite.tile.tile_id));
        self.polychrome_sprites
            .sort_by_key(|sprite| (sprite.order, sprite.tile.tile_id));
        self.surfaces.sort_by_key(|surface| surface.order);
    }

    #[cfg_attr(
        all(
            any(target_os = "linux", target_os = "freebsd"),
            not(any(feature = "x11", feature = "wayland"))
        ),
        allow(dead_code)
    )]
    pub(crate) fn batches(&self) -> impl Iterator<Item = PrimitiveBatch<'_>> {
        BatchIterator {
            backdrops: &self.backdrops,
            backdrops_start: 0,
            backdrops_iter: self.backdrops.iter().peekable(),
            shadows: &self.shadows,
            shadows_start: 0,
            shadows_iter: self.shadows.iter().peekable(),
            quads: &self.quads,
            quads_start: 0,
            quads_iter: self.quads.iter().peekable(),
            paths: &self.paths,
            paths_start: 0,
            paths_iter: self.paths.iter().peekable(),
            underlines: &self.underlines,
            underlines_start: 0,
            underlines_iter: self.underlines.iter().peekable(),
            monochrome_sprites: &self.monochrome_sprites,
            monochrome_sprites_start: 0,
            monochrome_sprites_iter: self.monochrome_sprites.iter().peekable(),
            polychrome_sprites: &self.polychrome_sprites,
            polychrome_sprites_start: 0,
            polychrome_sprites_iter: self.polychrome_sprites.iter().peekable(),
            surfaces: &self.surfaces,
            surfaces_start: 0,
            surfaces_iter: self.surfaces.iter().peekable(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Default)]
#[cfg_attr(
    all(
        any(target_os = "linux", target_os = "freebsd"),
        not(any(feature = "x11", feature = "wayland"))
    ),
    allow(dead_code)
)]
pub(crate) enum PrimitiveKind {
    Backdrop,
    Shadow,
    #[default]
    Quad,
    Path,
    Underline,
    MonochromeSprite,
    PolychromeSprite,
    Surface,
}

pub(crate) enum PaintOperation {
    Primitive(Primitive),
    StartLayer(Bounds<ScaledPixels>),
    EndLayer,
}

#[derive(Clone)]
pub(crate) enum Primitive {
    Backdrop(Backdrop),
    Shadow(Shadow),
    Quad(Quad),
    Path(Path<ScaledPixels>),
    Underline(Underline),
    MonochromeSprite(MonochromeSprite),
    PolychromeSprite(PolychromeSprite),
    Surface(PaintSurface),
}

impl Primitive {
    pub fn bounds(&self) -> &Bounds<ScaledPixels> {
        match self {
            Primitive::Backdrop(backdrop) => &backdrop.bounds,
            Primitive::Shadow(shadow) => &shadow.bounds,
            Primitive::Quad(quad) => &quad.bounds,
            Primitive::Path(path) => &path.bounds,
            Primitive::Underline(underline) => &underline.bounds,
            Primitive::MonochromeSprite(sprite) => &sprite.bounds,
            Primitive::PolychromeSprite(sprite) => &sprite.bounds,
            Primitive::Surface(surface) => &surface.bounds,
        }
    }

    pub fn content_mask(&self) -> &ContentMask<ScaledPixels> {
        match self {
            Primitive::Backdrop(backdrop) => &backdrop.content_mask,
            Primitive::Shadow(shadow) => &shadow.content_mask,
            Primitive::Quad(quad) => &quad.content_mask,
            Primitive::Path(path) => &path.content_mask,
            Primitive::Underline(underline) => &underline.content_mask,
            Primitive::MonochromeSprite(sprite) => &sprite.content_mask,
            Primitive::PolychromeSprite(sprite) => &sprite.content_mask,
            Primitive::Surface(surface) => &surface.content_mask,
        }
    }
}

#[cfg_attr(
    all(
        any(target_os = "linux", target_os = "freebsd"),
        not(any(feature = "x11", feature = "wayland"))
    ),
    allow(dead_code)
)]
struct BatchIterator<'a> {
    backdrops: &'a [Backdrop],
    backdrops_start: usize,
    backdrops_iter: Peekable<slice::Iter<'a, Backdrop>>,
    shadows: &'a [Shadow],
    shadows_start: usize,
    shadows_iter: Peekable<slice::Iter<'a, Shadow>>,
    quads: &'a [Quad],
    quads_start: usize,
    quads_iter: Peekable<slice::Iter<'a, Quad>>,
    paths: &'a [Path<ScaledPixels>],
    paths_start: usize,
    paths_iter: Peekable<slice::Iter<'a, Path<ScaledPixels>>>,
    underlines: &'a [Underline],
    underlines_start: usize,
    underlines_iter: Peekable<slice::Iter<'a, Underline>>,
    monochrome_sprites: &'a [MonochromeSprite],
    monochrome_sprites_start: usize,
    monochrome_sprites_iter: Peekable<slice::Iter<'a, MonochromeSprite>>,
    polychrome_sprites: &'a [PolychromeSprite],
    polychrome_sprites_start: usize,
    polychrome_sprites_iter: Peekable<slice::Iter<'a, PolychromeSprite>>,
    surfaces: &'a [PaintSurface],
    surfaces_start: usize,
    surfaces_iter: Peekable<slice::Iter<'a, PaintSurface>>,
}

impl<'a> Iterator for BatchIterator<'a> {
    type Item = PrimitiveBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut orders_and_kinds = [
            (
                self.backdrops_iter.peek().map(|backdrop| backdrop.order),
                PrimitiveKind::Backdrop,
            ),
            (
                self.shadows_iter.peek().map(|s| s.order),
                PrimitiveKind::Shadow,
            ),
            (self.quads_iter.peek().map(|q| q.order), PrimitiveKind::Quad),
            (self.paths_iter.peek().map(|q| q.order), PrimitiveKind::Path),
            (
                self.underlines_iter.peek().map(|u| u.order),
                PrimitiveKind::Underline,
            ),
            (
                self.monochrome_sprites_iter.peek().map(|s| s.order),
                PrimitiveKind::MonochromeSprite,
            ),
            (
                self.polychrome_sprites_iter.peek().map(|s| s.order),
                PrimitiveKind::PolychromeSprite,
            ),
            (
                self.surfaces_iter.peek().map(|s| s.order),
                PrimitiveKind::Surface,
            ),
        ];
        orders_and_kinds.sort_by_key(|(order, kind)| (order.unwrap_or(u32::MAX), *kind));

        let first = orders_and_kinds[0];
        let second = orders_and_kinds[1];
        let (batch_kind, max_order_and_kind) = if first.0.is_some() {
            (first.1, (second.0.unwrap_or(u32::MAX), second.1))
        } else {
            return None;
        };

        match batch_kind {
            PrimitiveKind::Backdrop => {
                let backdrops_start = self.backdrops_start;
                let mut backdrops_end = backdrops_start + 1;
                self.backdrops_iter.next();
                while self
                    .backdrops_iter
                    .next_if(|backdrop| (backdrop.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    backdrops_end += 1;
                }
                self.backdrops_start = backdrops_end;
                Some(PrimitiveBatch::Backdrops(
                    &self.backdrops[backdrops_start..backdrops_end],
                ))
            }
            PrimitiveKind::Shadow => {
                let shadows_start = self.shadows_start;
                let mut shadows_end = shadows_start + 1;
                self.shadows_iter.next();
                while self
                    .shadows_iter
                    .next_if(|shadow| (shadow.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    shadows_end += 1;
                }
                self.shadows_start = shadows_end;
                Some(PrimitiveBatch::Shadows(
                    &self.shadows[shadows_start..shadows_end],
                ))
            }
            PrimitiveKind::Quad => {
                let quads_start = self.quads_start;
                let mut quads_end = quads_start + 1;
                self.quads_iter.next();
                while self
                    .quads_iter
                    .next_if(|quad| (quad.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    quads_end += 1;
                }
                self.quads_start = quads_end;
                Some(PrimitiveBatch::Quads(&self.quads[quads_start..quads_end]))
            }
            PrimitiveKind::Path => {
                let paths_start = self.paths_start;
                let mut paths_end = paths_start + 1;
                self.paths_iter.next();
                while self
                    .paths_iter
                    .next_if(|path| (path.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    paths_end += 1;
                }
                self.paths_start = paths_end;
                Some(PrimitiveBatch::Paths(&self.paths[paths_start..paths_end]))
            }
            PrimitiveKind::Underline => {
                let underlines_start = self.underlines_start;
                let mut underlines_end = underlines_start + 1;
                self.underlines_iter.next();
                while self
                    .underlines_iter
                    .next_if(|underline| (underline.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    underlines_end += 1;
                }
                self.underlines_start = underlines_end;
                Some(PrimitiveBatch::Underlines(
                    &self.underlines[underlines_start..underlines_end],
                ))
            }
            PrimitiveKind::MonochromeSprite => {
                let texture_id = self.monochrome_sprites_iter.peek().unwrap().tile.texture_id;
                let sprites_start = self.monochrome_sprites_start;
                let mut sprites_end = sprites_start + 1;
                self.monochrome_sprites_iter.next();
                while self
                    .monochrome_sprites_iter
                    .next_if(|sprite| {
                        (sprite.order, batch_kind) < max_order_and_kind
                            && sprite.tile.texture_id == texture_id
                    })
                    .is_some()
                {
                    sprites_end += 1;
                }
                self.monochrome_sprites_start = sprites_end;
                Some(PrimitiveBatch::MonochromeSprites {
                    texture_id,
                    sprites: &self.monochrome_sprites[sprites_start..sprites_end],
                })
            }
            PrimitiveKind::PolychromeSprite => {
                let texture_id = self.polychrome_sprites_iter.peek().unwrap().tile.texture_id;
                let sprites_start = self.polychrome_sprites_start;
                let mut sprites_end = self.polychrome_sprites_start + 1;
                self.polychrome_sprites_iter.next();
                while self
                    .polychrome_sprites_iter
                    .next_if(|sprite| {
                        (sprite.order, batch_kind) < max_order_and_kind
                            && sprite.tile.texture_id == texture_id
                    })
                    .is_some()
                {
                    sprites_end += 1;
                }
                self.polychrome_sprites_start = sprites_end;
                Some(PrimitiveBatch::PolychromeSprites {
                    texture_id,
                    sprites: &self.polychrome_sprites[sprites_start..sprites_end],
                })
            }
            PrimitiveKind::Surface => {
                let surfaces_start = self.surfaces_start;
                let mut surfaces_end = surfaces_start + 1;
                self.surfaces_iter.next();
                while self
                    .surfaces_iter
                    .next_if(|surface| (surface.order, batch_kind) < max_order_and_kind)
                    .is_some()
                {
                    surfaces_end += 1;
                }
                self.surfaces_start = surfaces_end;
                Some(PrimitiveBatch::Surfaces(
                    &self.surfaces[surfaces_start..surfaces_end],
                ))
            }
        }
    }
}

#[derive(Debug)]
#[cfg_attr(
    all(
        any(target_os = "linux", target_os = "freebsd"),
        not(any(feature = "x11", feature = "wayland"))
    ),
    allow(dead_code)
)]
pub(crate) enum PrimitiveBatch<'a> {
    Backdrops(&'a [Backdrop]),
    Shadows(&'a [Shadow]),
    Quads(&'a [Quad]),
    Paths(&'a [Path<ScaledPixels>]),
    Underlines(&'a [Underline]),
    MonochromeSprites {
        texture_id: AtlasTextureId,
        sprites: &'a [MonochromeSprite],
    },
    PolychromeSprites {
        texture_id: AtlasTextureId,
        sprites: &'a [PolychromeSprite],
    },
    Surfaces(&'a [PaintSurface]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum BackdropPhase {
    Capture = 0,
    Composite = 1,
}

#[derive(Debug, Clone)]
pub(crate) struct Backdrop {
    pub order: DrawOrder,
    pub id: u32,
    pub phase: BackdropPhase,
    /// Both phases use the border box so filtering cannot read outside the element.
    pub bounds: Bounds<ScaledPixels>,
    /// Both phases clip to this mask before filtering and compositing.
    pub content_mask: ContentMask<ScaledPixels>,
    /// Composite rounds the real shape; Capture ignores these radii.
    pub corner_radii: Corners<ScaledPixels>,
    /// Capture's Gaussian standard deviation in device pixels.
    pub sigma: ScaledPixels,
    /// Composite's CSS saturation multiplier.
    pub saturation: f32,
    /// Composite applies element opacity to the filtered backdrop and tint as one group.
    pub opacity: f32,
    /// Composite compensates for the solid tint painted after it.
    pub tint_alpha: f32,
}

// Safety cap far above design values limits blur work from untrusted style inputs.
const MAX_BACKDROP_RADIUS: Pixels = px(200.);

impl Backdrop {
    pub(crate) fn pair(
        bounds: Bounds<Pixels>,
        corner_radii: Corners<Pixels>,
        content_mask: ContentMask<Pixels>,
        blur: BackdropBlur,
        scale_factor: f32,
        opacity: f32,
    ) -> Option<(Self, Self)> {
        if !f32::from(blur.radius).is_finite() || blur.radius <= Pixels::ZERO {
            return None;
        }
        if opacity.is_nan() || opacity <= 0. || bounds.intersect(&content_mask.bounds).is_empty() {
            return None;
        }
        let saturation = if blur.saturation.is_finite() {
            blur.saturation.max(0.)
        } else {
            1.
        };
        let radius = blur.radius.min(MAX_BACKDROP_RADIUS);
        let composite = Self {
            order: 0,
            id: u32::MAX,
            phase: BackdropPhase::Composite,
            bounds: bounds.scale(scale_factor),
            content_mask: content_mask.scale(scale_factor),
            corner_radii: corner_radii.scale(scale_factor),
            sigma: radius.scale(scale_factor),
            saturation,
            opacity: opacity.min(1.),
            tint_alpha: 0.,
        };
        let capture = Self {
            phase: BackdropPhase::Capture,
            content_mask: composite.content_mask.clone(),
            ..composite
        };
        Some((capture, composite))
    }
}

impl From<Backdrop> for Primitive {
    fn from(backdrop: Backdrop) -> Self {
        Primitive::Backdrop(backdrop)
    }
}

#[repr(C)]
pub(crate) struct BackdropUniforms {
    pub bounds: Bounds<ScaledPixels>,
    pub clip_bounds: Bounds<ScaledPixels>,
    pub corner_radii: Corners<ScaledPixels>,
    pub sample_origin: Point<f32>,
    pub saturation: f32,
    pub opacity: f32,
    pub tint_alpha: f32,
    pub padding: [f32; 3],
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub(crate) struct Quad {
    pub order: DrawOrder,
    pub border_style: BorderStyle,
    pub bounds: Bounds<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
    pub background: Background,
    pub border_color: Hsla,
    pub corner_radii: Corners<ScaledPixels>,
    pub border_widths: Edges<ScaledPixels>,
}

impl From<Quad> for Primitive {
    fn from(quad: Quad) -> Self {
        Primitive::Quad(quad)
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct Underline {
    pub order: DrawOrder,
    pub pad: u32, // align to 8 bytes
    pub bounds: Bounds<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
    pub color: Hsla,
    pub thickness: ScaledPixels,
    pub wavy: u32,
}

impl From<Underline> for Primitive {
    fn from(underline: Underline) -> Self {
        Primitive::Underline(underline)
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct Shadow {
    pub order: DrawOrder,
    pub blur_radius: ScaledPixels,
    pub bounds: Bounds<ScaledPixels>,
    pub corner_radii: Corners<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
    pub color: Hsla,
}

impl From<Shadow> for Primitive {
    fn from(shadow: Shadow) -> Self {
        Primitive::Shadow(shadow)
    }
}

/// The style of a border.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[repr(C)]
pub enum BorderStyle {
    /// A solid border.
    #[default]
    Solid = 0,
    /// A dashed border.
    Dashed = 1,
}

/// A data type representing a 2 dimensional transformation that can be applied to an element.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct TransformationMatrix {
    /// 2x2 matrix containing rotation and scale,
    /// stored row-major
    pub rotation_scale: [[f32; 2]; 2],
    /// translation vector
    pub translation: [f32; 2],
}

impl Eq for TransformationMatrix {}

impl TransformationMatrix {
    /// The unit matrix, has no effect.
    pub fn unit() -> Self {
        Self {
            rotation_scale: [[1.0, 0.0], [0.0, 1.0]],
            translation: [0.0, 0.0],
        }
    }

    /// Move the origin by a given point
    pub fn translate(mut self, point: Point<ScaledPixels>) -> Self {
        self.compose(Self {
            rotation_scale: [[1.0, 0.0], [0.0, 1.0]],
            translation: [point.x.0, point.y.0],
        })
    }

    /// Clockwise rotation in radians around the origin
    pub fn rotate(self, angle: Radians) -> Self {
        self.compose(Self {
            rotation_scale: [
                [angle.0.cos(), -angle.0.sin()],
                [angle.0.sin(), angle.0.cos()],
            ],
            translation: [0.0, 0.0],
        })
    }

    /// Scale around the origin
    pub fn scale(self, size: Size<f32>) -> Self {
        self.compose(Self {
            rotation_scale: [[size.width, 0.0], [0.0, size.height]],
            translation: [0.0, 0.0],
        })
    }

    /// Perform matrix multiplication with another transformation
    /// to produce a new transformation that is the result of
    /// applying both transformations: first, `other`, then `self`.
    #[inline]
    pub fn compose(self, other: TransformationMatrix) -> TransformationMatrix {
        if other == Self::unit() {
            return self;
        }
        // Perform matrix multiplication
        TransformationMatrix {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * other.rotation_scale[0][0]
                        + self.rotation_scale[0][1] * other.rotation_scale[1][0],
                    self.rotation_scale[0][0] * other.rotation_scale[0][1]
                        + self.rotation_scale[0][1] * other.rotation_scale[1][1],
                ],
                [
                    self.rotation_scale[1][0] * other.rotation_scale[0][0]
                        + self.rotation_scale[1][1] * other.rotation_scale[1][0],
                    self.rotation_scale[1][0] * other.rotation_scale[0][1]
                        + self.rotation_scale[1][1] * other.rotation_scale[1][1],
                ],
            ],
            translation: [
                self.translation[0]
                    + self.rotation_scale[0][0] * other.translation[0]
                    + self.rotation_scale[0][1] * other.translation[1],
                self.translation[1]
                    + self.rotation_scale[1][0] * other.translation[0]
                    + self.rotation_scale[1][1] * other.translation[1],
            ],
        }
    }

    /// Apply transformation to a point, mainly useful for debugging
    pub fn apply(&self, point: Point<Pixels>) -> Point<Pixels> {
        let input = [point.x.0, point.y.0];
        let mut output = self.translation;
        for (i, output_cell) in output.iter_mut().enumerate() {
            for (k, input_cell) in input.iter().enumerate() {
                *output_cell += self.rotation_scale[i][k] * *input_cell;
            }
        }
        Point::new(output[0].into(), output[1].into())
    }
}

impl Default for TransformationMatrix {
    fn default() -> Self {
        Self::unit()
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub(crate) struct MonochromeSprite {
    pub order: DrawOrder,
    pub pad: u32, // align to 8 bytes
    pub bounds: Bounds<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
    pub color: Hsla,
    pub tile: AtlasTile,
    pub transformation: TransformationMatrix,
}

impl From<MonochromeSprite> for Primitive {
    fn from(sprite: MonochromeSprite) -> Self {
        Primitive::MonochromeSprite(sprite)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub(crate) struct PolychromeSprite {
    pub order: DrawOrder,
    pub pad: u32, // align to 8 bytes
    pub grayscale: bool,
    pub opacity: f32,
    pub bounds: Bounds<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
    pub corner_radii: Corners<ScaledPixels>,
    pub tile: AtlasTile,
}

impl From<PolychromeSprite> for Primitive {
    fn from(sprite: PolychromeSprite) -> Self {
        Primitive::PolychromeSprite(sprite)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PaintSurface {
    pub order: DrawOrder,
    pub bounds: Bounds<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
    #[cfg(target_os = "macos")]
    pub image_buffer: core_video::pixel_buffer::CVPixelBuffer,
}

impl From<PaintSurface> for Primitive {
    fn from(surface: PaintSurface) -> Self {
        Primitive::Surface(surface)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PathId(pub(crate) usize);

/// A line made up of a series of vertices and control points.
#[derive(Clone, Debug)]
pub struct Path<P: Clone + Debug + Default + PartialEq> {
    pub(crate) id: PathId,
    pub(crate) order: DrawOrder,
    pub(crate) bounds: Bounds<P>,
    pub(crate) content_mask: ContentMask<P>,
    pub(crate) vertices: Vec<PathVertex<P>>,
    pub(crate) color: Background,
    start: Point<P>,
    current: Point<P>,
    contour_count: usize,
}

impl Path<Pixels> {
    /// Create a new path with the given starting point.
    pub fn new(start: Point<Pixels>) -> Self {
        Self {
            id: PathId(0),
            order: DrawOrder::default(),
            vertices: Vec::new(),
            start,
            current: start,
            bounds: Bounds {
                origin: start,
                size: Default::default(),
            },
            content_mask: Default::default(),
            color: Default::default(),
            contour_count: 0,
        }
    }

    /// Scale this path by the given factor.
    pub fn scale(&self, factor: f32) -> Path<ScaledPixels> {
        Path {
            id: self.id,
            order: self.order,
            bounds: self.bounds.scale(factor),
            content_mask: self.content_mask.scale(factor),
            vertices: self
                .vertices
                .iter()
                .map(|vertex| vertex.scale(factor))
                .collect(),
            start: self.start.map(|start| start.scale(factor)),
            current: self.current.scale(factor),
            contour_count: self.contour_count,
            color: self.color,
        }
    }

    /// Move the start, current point to the given point.
    pub fn move_to(&mut self, to: Point<Pixels>) {
        self.contour_count += 1;
        self.start = to;
        self.current = to;
    }

    /// Draw a straight line from the current point to the given point.
    pub fn line_to(&mut self, to: Point<Pixels>) {
        self.contour_count += 1;
        if self.contour_count > 1 {
            self.push_triangle(
                (self.start, self.current, to),
                (point(0., 1.), point(0., 1.), point(0., 1.)),
            );
        }
        self.current = to;
    }

    /// Draw a curve from the current point to the given point, using the given control point.
    pub fn curve_to(&mut self, to: Point<Pixels>, ctrl: Point<Pixels>) {
        self.contour_count += 1;
        if self.contour_count > 1 {
            self.push_triangle(
                (self.start, self.current, to),
                (point(0., 1.), point(0., 1.), point(0., 1.)),
            );
        }

        self.push_triangle(
            (self.current, ctrl, to),
            (point(0., 0.), point(0.5, 0.), point(1., 1.)),
        );
        self.current = to;
    }

    /// Push a triangle to the Path.
    pub fn push_triangle(
        &mut self,
        xy: (Point<Pixels>, Point<Pixels>, Point<Pixels>),
        st: (Point<f32>, Point<f32>, Point<f32>),
    ) {
        self.bounds = self
            .bounds
            .union(&Bounds {
                origin: xy.0,
                size: Default::default(),
            })
            .union(&Bounds {
                origin: xy.1,
                size: Default::default(),
            })
            .union(&Bounds {
                origin: xy.2,
                size: Default::default(),
            });

        self.vertices.push(PathVertex {
            xy_position: xy.0,
            st_position: st.0,
            content_mask: Default::default(),
        });
        self.vertices.push(PathVertex {
            xy_position: xy.1,
            st_position: st.1,
            content_mask: Default::default(),
        });
        self.vertices.push(PathVertex {
            xy_position: xy.2,
            st_position: st.2,
            content_mask: Default::default(),
        });
    }
}

impl<T> Path<T>
where
    T: Clone + Debug + Default + PartialEq + PartialOrd + Add<T, Output = T> + Sub<Output = T>,
{
    #[allow(unused)]
    pub(crate) fn clipped_bounds(&self) -> Bounds<T> {
        self.bounds.intersect(&self.content_mask.bounds)
    }
}

impl From<Path<ScaledPixels>> for Primitive {
    fn from(path: Path<ScaledPixels>) -> Self {
        Primitive::Path(path)
    }
}

#[cfg(test)]
mod backdrop_tests {
    use super::*;
    use crate::{BackdropBlur, px, size};

    fn backdrop(phase: BackdropPhase) -> Backdrop {
        let bounds = Bounds::new(
            point(ScaledPixels(10.), ScaledPixels(10.)),
            size(ScaledPixels(80.), ScaledPixels(80.)),
        );
        Backdrop {
            order: 0,
            id: u32::MAX,
            phase,
            bounds,
            content_mask: ContentMask { bounds },
            corner_radii: Corners::all(ScaledPixels(10.)),
            sigma: ScaledPixels(8.),
            saturation: 1.,
            opacity: 1.,
            tint_alpha: 0.,
        }
    }

    #[test]
    fn offset_shadow_stays_between_capture_and_composite() {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(40.)));
        let shadow = crate::BoxShadow {
            color: crate::black(),
            offset: point(px(0.), px(60.)),
            spread_radius: px(0.),
            blur_radius: px(30.),
        };
        let footprint = crate::style::shadow_footprint(bounds, std::slice::from_ref(&shadow));
        let (capture, composite) = Backdrop::pair(
            bounds,
            Corners::default(),
            ContentMask { bounds: footprint },
            BackdropBlur {
                radius: px(8.),
                saturation: 1.,
            },
            1.,
            1.,
        )
        .unwrap();
        let mut scene = Scene::default();
        scene.insert_primitive(capture);
        scene.push_layer(footprint.scale(1.));
        scene.insert_primitive(Shadow {
            order: 0,
            bounds: (bounds + shadow.offset)
                .dilate(shadow.spread_radius)
                .scale(1.),
            blur_radius: shadow.blur_radius.scale(1.),
            corner_radii: Corners::default(),
            content_mask: ContentMask {
                bounds: footprint.scale(1.),
            },
            color: shadow.color,
        });
        scene.pop_layer();
        scene.insert_primitive(composite);
        let mut replay = Scene::default();
        replay.replay(0..scene.len(), &scene);
        for mut ordered_scene in [scene, replay] {
            ordered_scene.finish();
            assert!(ordered_scene.backdrops[0].order < ordered_scene.shadows[0].order);
            assert!(ordered_scene.shadows[0].order < ordered_scene.backdrops[1].order);
        }
    }

    #[test]
    fn clipped_composite_keeps_ids_paired_on_replay() {
        let mut scene = Scene::default();
        let bounds = Bounds::new(point(px(10.), px(10.)), size(px(80.), px(80.)));
        let (outer_capture, outer_composite) = Backdrop::pair(
            bounds,
            Corners::all(px(10.)),
            ContentMask { bounds },
            BackdropBlur {
                radius: px(8.),
                saturation: 1.,
            },
            1.,
            1.,
        )
        .unwrap();
        let inner_bounds = Bounds::new(point(px(85.), px(10.)), size(px(80.), px(80.)));
        let (inner_capture, mut inner_composite) = Backdrop::pair(
            inner_bounds,
            Corners::all(px(10.)),
            ContentMask { bounds },
            BackdropBlur {
                radius: px(8.),
                saturation: 1.,
            },
            1.,
            1.,
        )
        .unwrap();
        // Replay must preserve pairing even when only the composite becomes clipped.
        inner_composite.content_mask.bounds.size.width = ScaledPixels(75.);
        scene.insert_primitive(outer_capture);
        scene.insert_primitive(inner_capture);
        scene.insert_primitive(inner_composite);
        scene.insert_primitive(outer_composite);
        assert_eq!(scene.backdrops.last().unwrap().id, 0);
        assert!(scene.backdrop_stack.is_empty());

        let mut replay = Scene::default();
        replay.replay(0..scene.len(), &scene);
        assert_eq!(replay.backdrops.last().unwrap().id, 0);
        assert!(replay.backdrop_stack.is_empty());
        assert_eq!(replay.len(), 4);
    }

    #[test]
    fn backdrop_pair_scales_and_rejects_invalid_input() {
        let bounds = Bounds::new(point(px(10.), px(20.)), size(px(100.), px(80.)));
        let mask = ContentMask { bounds };
        let corners = Corners::all(px(12.));
        let blur = BackdropBlur {
            radius: px(30.),
            saturation: 1.5,
        };
        let (capture, composite) =
            Backdrop::pair(bounds, corners, mask.clone(), blur, 2., 0.5).unwrap();
        assert_eq!(capture.sigma, ScaledPixels(60.));
        assert_eq!(
            composite.bounds,
            Bounds::new(
                point(ScaledPixels(20.), ScaledPixels(40.)),
                size(ScaledPixels(200.), ScaledPixels(160.)),
            )
        );
        assert_eq!(capture.bounds, composite.bounds);
        assert_eq!(capture.content_mask.bounds, composite.content_mask.bounds);
        assert_eq!(composite.corner_radii, Corners::all(ScaledPixels(24.)));
        assert_eq!(composite.saturation, 1.5);
        assert_eq!(composite.opacity, 0.5);
        assert_eq!(capture.phase, BackdropPhase::Capture);
        assert_eq!(composite.phase, BackdropPhase::Composite);
        for radius in [0., -1., f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            assert!(
                Backdrop::pair(
                    bounds,
                    corners,
                    mask.clone(),
                    BackdropBlur {
                        radius: px(radius),
                        ..blur
                    },
                    2.,
                    1.
                )
                .is_none()
            );
        }
        assert!(Backdrop::pair(bounds, corners, mask.clone(), blur, 2., 0.).is_none());
        assert!(Backdrop::pair(bounds, corners, mask.clone(), blur, 2., f32::NAN).is_none());
        let (_, composite) = Backdrop::pair(bounds, corners, mask.clone(), blur, 2., 1.5).unwrap();
        assert_eq!(composite.opacity, 1.);
        let (capture, composite) = Backdrop::pair(
            bounds,
            corners,
            mask.clone(),
            BackdropBlur {
                radius: px(10_000.),
                ..blur
            },
            2.,
            1.,
        )
        .unwrap();
        assert_eq!(capture.sigma, ScaledPixels(400.));
        assert_eq!(capture.bounds, composite.bounds);
        let empty_mask = ContentMask {
            bounds: Bounds::default(),
        };
        assert!(Backdrop::pair(bounds, corners, empty_mask, blur, 2., 1.).is_none());
        for (saturation, expected) in [(f32::NAN, 1.), (f32::INFINITY, 1.), (-1., 0.)] {
            let (_, composite) = Backdrop::pair(
                bounds,
                corners,
                mask.clone(),
                BackdropBlur { saturation, ..blur },
                2.,
                1.,
            )
            .unwrap();
            assert_eq!(composite.saturation, expected);
        }
        assert_eq!(BackdropBlur::default().radius, Pixels::ZERO);
        assert_eq!(BackdropBlur::default().saturation, 1.);
    }

    #[test]
    fn backdrop_ids_pair_under_nesting_and_replay() {
        let mut scene = Scene::default();
        scene.insert_primitive(backdrop(BackdropPhase::Capture));
        scene.insert_primitive(backdrop(BackdropPhase::Capture));
        scene.insert_primitive(backdrop(BackdropPhase::Composite));
        scene.insert_primitive(backdrop(BackdropPhase::Composite));
        assert_eq!(
            scene
                .backdrops
                .iter()
                .map(|backdrop| backdrop.id)
                .collect::<Vec<_>>(),
            [0, 1, 1, 0]
        );

        let mut replay = Scene::default();
        replay.replay(0..scene.len(), &scene);
        assert_eq!(
            replay
                .backdrops
                .iter()
                .map(|backdrop| backdrop.id)
                .collect::<Vec<_>>(),
            [0, 1, 1, 0]
        );
        replay.clear();
        replay.insert_primitive(backdrop(BackdropPhase::Capture));
        replay.insert_primitive(backdrop(BackdropPhase::Composite));
        replay.replay(0..scene.len(), &scene);
        assert_eq!(
            replay
                .backdrops
                .iter()
                .map(|backdrop| backdrop.id)
                .collect::<Vec<_>>(),
            [0, 0, 1, 2, 2, 1]
        );

        scene.clear();
        scene.insert_primitive(backdrop(BackdropPhase::Capture));
        scene.insert_primitive(backdrop(BackdropPhase::Composite));
        assert_eq!(
            scene
                .backdrops
                .iter()
                .map(|backdrop| backdrop.id)
                .collect::<Vec<_>>(),
            [0, 0]
        );
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub(crate) struct PathVertex<P: Clone + Debug + Default + PartialEq> {
    pub(crate) xy_position: Point<P>,
    pub(crate) st_position: Point<f32>,
    pub(crate) content_mask: ContentMask<P>,
}

impl PathVertex<Pixels> {
    pub fn scale(&self, factor: f32) -> PathVertex<ScaledPixels> {
        PathVertex {
            xy_position: self.xy_position.scale(factor),
            st_position: self.st_position,
            content_mask: self.content_mask.scale(factor),
        }
    }
}
