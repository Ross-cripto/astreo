// Modified for Astreo: backdrop blur (see PATCHES.md).
use super::metal_atlas::MetalAtlas;
use crate::{
    AtlasTextureId, Backdrop, BackdropPhase, BackdropUniforms, Background, Bounds, ContentMask,
    DevicePixels, MonochromeSprite, PaintSurface, Path, Point, PolychromeSprite, PrimitiveBatch,
    Quad, ScaledPixels, Scene, Shadow, Size, Surface, Underline, point, size,
};
use anyhow::Result;
use block::ConcreteBlock;
use cocoa::{
    base::{NO, YES, nil},
    foundation::{NSProcessInfo, NSSize, NSUInteger},
    quartzcore::AutoresizingMask,
};

use core_foundation::base::TCFType;
use core_video::{
    metal_texture::CVMetalTextureGetTexture, metal_texture_cache::CVMetalTextureCache,
    pixel_buffer::kCVPixelFormatType_420YpCbCr8BiPlanarFullRange,
};
use foreign_types::{ForeignType, ForeignTypeRef};
use metal::{
    CAMetalLayer, CommandQueue, MTLPixelFormat, MTLResourceOptions, NSRange,
    RenderPassColorAttachmentDescriptorRef,
};
use objc::{self, msg_send, sel, sel_impl};
use parking_lot::Mutex;

use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
    sync::Arc,
};

// Exported to metal
pub(crate) type PointF = crate::Point<f32>;

#[cfg(not(feature = "runtime_shaders"))]
const SHADERS_METALLIB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shaders.metallib"));
#[cfg(feature = "runtime_shaders")]
const SHADERS_SOURCE_FILE: &str = include_str!(concat!(env!("OUT_DIR"), "/stitched_shaders.metal"));
// Use 4x MSAA, all devices support it.
// https://developer.apple.com/documentation/metal/mtldevice/1433355-supportstexturesamplecount
const PATH_SAMPLE_COUNT: u32 = 4;
const MPS_IMAGE_EDGE_MODE_CLAMP: usize = 1;
const MPS_IMAGE_EDGE_MODE_MIRROR_WITH_EDGE: usize = 3;
const _: () = assert!(std::mem::size_of::<BackdropUniforms>() == 80);

unsafe extern "C" {
    fn MPSSupportsMTLDevice(device: *mut c_void) -> objc::runtime::BOOL;
}

pub type Context = Arc<Mutex<InstanceBufferPool>>;
pub type Renderer = MetalRenderer;

pub unsafe fn new_renderer(
    context: self::Context,
    _native_window: *mut c_void,
    _native_view: *mut c_void,
    _bounds: crate::Size<f32>,
    _transparent: bool,
) -> Renderer {
    MetalRenderer::new(context)
}

pub(crate) struct InstanceBufferPool {
    buffer_size: usize,
    buffers: Vec<metal::Buffer>,
}

impl Default for InstanceBufferPool {
    fn default() -> Self {
        Self {
            buffer_size: 2 * 1024 * 1024,
            buffers: Vec::new(),
        }
    }
}

pub(crate) struct InstanceBuffer {
    metal_buffer: metal::Buffer,
    size: usize,
}

impl InstanceBufferPool {
    pub(crate) fn reset(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        self.buffers.clear();
    }

    pub(crate) fn acquire(&mut self, device: &metal::Device) -> InstanceBuffer {
        let buffer = self.buffers.pop().unwrap_or_else(|| {
            device.new_buffer(
                self.buffer_size as u64,
                MTLResourceOptions::StorageModeManaged,
            )
        });
        InstanceBuffer {
            metal_buffer: buffer,
            size: self.buffer_size,
        }
    }

    pub(crate) fn release(&mut self, buffer: InstanceBuffer) {
        if buffer.size == self.buffer_size {
            self.buffers.push(buffer.metal_buffer)
        }
    }
}

pub(crate) struct MetalRenderer {
    device: metal::Device,
    layer: metal::MetalLayer,
    presents_with_transaction: bool,
    command_queue: CommandQueue,
    paths_rasterization_pipeline_state: metal::RenderPipelineState,
    path_sprites_pipeline_state: metal::RenderPipelineState,
    shadows_pipeline_state: metal::RenderPipelineState,
    quads_pipeline_state: metal::RenderPipelineState,
    backdrop_composite_pipeline_state: metal::RenderPipelineState,
    blur_kernels: HashMap<u32, BlurKernel>,
    blur_supported: bool,
    blur_edge_mode: usize,
    blur_unsupported_logged: bool,
    blur_initialization_failures: HashSet<u32>,
    unmatched_composite_logged: Cell<bool>,
    backdrop_pool: Vec<metal::Texture>,
    backdrop_frame: Vec<Option<BackdropCapture>>,
    underlines_pipeline_state: metal::RenderPipelineState,
    monochrome_sprites_pipeline_state: metal::RenderPipelineState,
    polychrome_sprites_pipeline_state: metal::RenderPipelineState,
    surfaces_pipeline_state: metal::RenderPipelineState,
    unit_vertices: metal::Buffer,
    #[allow(clippy::arc_with_non_send_sync)]
    instance_buffer_pool: Arc<Mutex<InstanceBufferPool>>,
    sprite_atlas: Arc<MetalAtlas>,
    core_video_texture_cache: core_video::metal_texture_cache::CVMetalTextureCache,
    path_intermediate_texture: Option<metal::Texture>,
    path_intermediate_msaa_texture: Option<metal::Texture>,
    path_sample_count: u32,
}

struct BackdropCapture {
    blurred: metal::Texture,
    sample: metal::Texture,
    origin: Point<DevicePixels>,
}

struct BlurKernel(NonNull<objc::runtime::Object>);

impl Drop for BlurKernel {
    fn drop(&mut self) {
        // initWithDevice:sigma: transfers one retain to this owner.
        unsafe {
            let _: () = msg_send![self.0.as_ptr(), release];
        }
    }
}

#[repr(C)]
pub struct PathRasterizationVertex {
    pub xy_position: Point<ScaledPixels>,
    pub st_position: Point<f32>,
    pub color: Background,
    pub bounds: Bounds<ScaledPixels>,
}

impl MetalRenderer {
    pub fn new(instance_buffer_pool: Arc<Mutex<InstanceBufferPool>>) -> Self {
        // Prefer low‐power integrated GPUs on Intel Mac. On Apple
        // Silicon, there is only ever one GPU, so this is equivalent to
        // `metal::Device::system_default()`.
        let mut devices = metal::Device::all();
        devices.sort_by_key(|device| (device.is_removable(), device.is_low_power()));
        let Some(device) = devices.pop() else {
            log::error!("unable to access a compatible graphics device");
            std::process::exit(1);
        };

        let blur_supported = (unsafe { MPSSupportsMTLDevice(device.as_ptr().cast()) } != NO)
            && objc::runtime::Class::get("MPSImageGaussianBlur").is_some();
        let layer = metal::MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_opaque(false);
        layer.set_maximum_drawable_count(3);
        unsafe {
            let _: () = msg_send![&*layer, setAllowsNextDrawableTimeout: NO];
            let _: () = msg_send![&*layer, setNeedsDisplayOnBoundsChange: YES];
            let _: () = msg_send![
                &*layer,
                setAutoresizingMask: AutoresizingMask::WIDTH_SIZABLE
                    | AutoresizingMask::HEIGHT_SIZABLE
            ];
        }
        #[cfg(feature = "runtime_shaders")]
        let library = device
            .new_library_with_source(&SHADERS_SOURCE_FILE, &metal::CompileOptions::new())
            .expect("error building metal library");
        #[cfg(not(feature = "runtime_shaders"))]
        let library = device
            .new_library_with_data(SHADERS_METALLIB)
            .expect("error building metal library");

        fn to_float2_bits(point: PointF) -> u64 {
            let mut output = point.y.to_bits() as u64;
            output <<= 32;
            output |= point.x.to_bits() as u64;
            output
        }

        let unit_vertices = [
            to_float2_bits(point(0., 0.)),
            to_float2_bits(point(1., 0.)),
            to_float2_bits(point(0., 1.)),
            to_float2_bits(point(0., 1.)),
            to_float2_bits(point(1., 0.)),
            to_float2_bits(point(1., 1.)),
        ];
        let unit_vertices = device.new_buffer_with_data(
            unit_vertices.as_ptr() as *const c_void,
            mem::size_of_val(&unit_vertices) as u64,
            MTLResourceOptions::StorageModeManaged,
        );

        let paths_rasterization_pipeline_state = build_path_rasterization_pipeline_state(
            &device,
            &library,
            "paths_rasterization",
            "path_rasterization_vertex",
            "path_rasterization_fragment",
            MTLPixelFormat::BGRA8Unorm,
            PATH_SAMPLE_COUNT,
        );
        let path_sprites_pipeline_state = build_path_sprite_pipeline_state(
            &device,
            &library,
            "path_sprites",
            "path_sprite_vertex",
            "path_sprite_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );
        let shadows_pipeline_state = build_pipeline_state(
            &device,
            &library,
            "shadows",
            "shadow_vertex",
            "shadow_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );
        let quads_pipeline_state = build_pipeline_state(
            &device,
            &library,
            "quads",
            "quad_vertex",
            "quad_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );
        let underlines_pipeline_state = build_pipeline_state(
            &device,
            &library,
            "underlines",
            "underline_vertex",
            "underline_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );
        let monochrome_sprites_pipeline_state = build_pipeline_state(
            &device,
            &library,
            "monochrome_sprites",
            "monochrome_sprite_vertex",
            "monochrome_sprite_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );
        let polychrome_sprites_pipeline_state = build_pipeline_state(
            &device,
            &library,
            "polychrome_sprites",
            "polychrome_sprite_vertex",
            "polychrome_sprite_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );
        let surfaces_pipeline_state = build_pipeline_state(
            &device,
            &library,
            "surfaces",
            "surface_vertex",
            "surface_fragment",
            MTLPixelFormat::BGRA8Unorm,
        );

        let backdrop_composite_pipeline_state = build_backdrop_pipeline_state(&device, &library);
        let command_queue = device.new_command_queue();
        let sprite_atlas = Arc::new(MetalAtlas::new(device.clone()));
        let core_video_texture_cache =
            CVMetalTextureCache::new(None, device.clone(), None).unwrap();

        Self {
            device,
            layer,
            presents_with_transaction: false,
            command_queue,
            paths_rasterization_pipeline_state,
            path_sprites_pipeline_state,
            shadows_pipeline_state,
            quads_pipeline_state,
            backdrop_composite_pipeline_state,
            blur_kernels: HashMap::new(),
            blur_supported,
            blur_edge_mode: unsafe {
                let process_info = NSProcessInfo::processInfo(nil);
                if process_info.operatingSystemVersion().majorVersion >= 13 {
                    MPS_IMAGE_EDGE_MODE_MIRROR_WITH_EDGE
                } else {
                    MPS_IMAGE_EDGE_MODE_CLAMP
                }
            },
            blur_unsupported_logged: false,
            blur_initialization_failures: HashSet::new(),
            unmatched_composite_logged: Cell::new(false),
            backdrop_pool: Vec::new(),
            backdrop_frame: Vec::new(),
            underlines_pipeline_state,
            monochrome_sprites_pipeline_state,
            polychrome_sprites_pipeline_state,
            surfaces_pipeline_state,
            unit_vertices,
            instance_buffer_pool,
            sprite_atlas,
            core_video_texture_cache,
            path_intermediate_texture: None,
            path_intermediate_msaa_texture: None,
            path_sample_count: PATH_SAMPLE_COUNT,
        }
    }

    pub fn layer(&self) -> &metal::MetalLayerRef {
        &self.layer
    }

    pub fn layer_ptr(&self) -> *mut CAMetalLayer {
        self.layer.as_ptr()
    }

    pub fn sprite_atlas(&self) -> &Arc<MetalAtlas> {
        &self.sprite_atlas
    }

    pub fn set_presents_with_transaction(&mut self, presents_with_transaction: bool) {
        self.presents_with_transaction = presents_with_transaction;
        self.layer
            .set_presents_with_transaction(presents_with_transaction);
    }

    pub fn update_drawable_size(&mut self, size: Size<DevicePixels>) {
        let size = NSSize {
            width: size.width.0 as f64,
            height: size.height.0 as f64,
        };
        unsafe {
            let _: () = msg_send![
                self.layer(),
                setDrawableSize: size
            ];
        }
        let device_pixels_size = Size {
            width: DevicePixels(size.width as i32),
            height: DevicePixels(size.height as i32),
        };
        self.update_path_intermediate_textures(device_pixels_size);
    }

    fn update_path_intermediate_textures(&mut self, size: Size<DevicePixels>) {
        // We are uncertain when this happens, but sometimes size can be 0 here. Most likely before
        // the layout pass on window creation. Zero-sized texture creation causes SIGABRT.
        // https://github.com/zed-industries/zed/issues/36229
        if size.width.0 <= 0 || size.height.0 <= 0 {
            self.path_intermediate_texture = None;
            self.path_intermediate_msaa_texture = None;
            return;
        }

        let texture_descriptor = metal::TextureDescriptor::new();
        texture_descriptor.set_width(size.width.0 as u64);
        texture_descriptor.set_height(size.height.0 as u64);
        texture_descriptor.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        texture_descriptor
            .set_usage(metal::MTLTextureUsage::RenderTarget | metal::MTLTextureUsage::ShaderRead);
        self.path_intermediate_texture = Some(self.device.new_texture(&texture_descriptor));

        if self.path_sample_count > 1 {
            let mut msaa_descriptor = texture_descriptor;
            msaa_descriptor.set_texture_type(metal::MTLTextureType::D2Multisample);
            msaa_descriptor.set_storage_mode(metal::MTLStorageMode::Private);
            msaa_descriptor.set_sample_count(self.path_sample_count as _);
            self.path_intermediate_msaa_texture = Some(self.device.new_texture(&msaa_descriptor));
        } else {
            self.path_intermediate_msaa_texture = None;
        }
    }

    pub fn update_transparency(&self, _transparent: bool) {
        // todo(mac)?
    }

    pub fn destroy(&self) {
        // nothing to do
    }

    fn sync_framebuffer_only(&self, framebuffer_only: bool) {
        if self.layer.framebuffer_only() != framebuffer_only {
            self.layer.set_framebuffer_only(framebuffer_only);
        }
    }

    fn prepare_layer(&self, scene: &Scene) {
        self.sync_framebuffer_only(scene.backdrops.is_empty());
    }

    pub fn draw(&mut self, scene: &Scene) {
        let layer = self.layer.clone();
        let viewport_size = layer.drawable_size();
        let viewport_size: Size<DevicePixels> = size(
            (viewport_size.width.ceil() as i32).into(),
            (viewport_size.height.ceil() as i32).into(),
        );
        self.prepare_layer(scene);
        let drawable = if let Some(drawable) = layer.next_drawable() {
            drawable
        } else {
            log::error!(
                "failed to retrieve next drawable, drawable size: {:?}",
                viewport_size
            );
            return;
        };

        loop {
            let mut instance_buffer = self.instance_buffer_pool.lock().acquire(&self.device);

            let command_buffer = self.draw_primitives(
                scene,
                &mut instance_buffer,
                drawable.texture(),
                viewport_size,
            );

            match command_buffer {
                Ok(command_buffer) => {
                    let instance_buffer_pool = self.instance_buffer_pool.clone();
                    let instance_buffer = Cell::new(Some(instance_buffer));
                    let block = ConcreteBlock::new(move |_| {
                        if let Some(instance_buffer) = instance_buffer.take() {
                            instance_buffer_pool.lock().release(instance_buffer);
                        }
                    });
                    let block = block.copy();
                    command_buffer.add_completed_handler(&block);

                    if self.presents_with_transaction {
                        command_buffer.commit();
                        command_buffer.wait_until_scheduled();
                        drawable.present();
                    } else {
                        command_buffer.present_drawable(drawable);
                        command_buffer.commit();
                    }
                    return;
                }
                Err(err) => {
                    log::error!(
                        "failed to render: {}. retrying with larger instance buffer size",
                        err
                    );
                    let mut instance_buffer_pool = self.instance_buffer_pool.lock();
                    let buffer_size = instance_buffer_pool.buffer_size;
                    if buffer_size >= 256 * 1024 * 1024 {
                        log::error!("instance buffer size grew too large: {}", buffer_size);
                        break;
                    }
                    instance_buffer_pool.reset(buffer_size * 2);
                    log::info!(
                        "increased instance buffer size to {}",
                        instance_buffer_pool.buffer_size
                    );
                }
            }
        }
    }

    fn draw_primitives(
        &mut self,
        scene: &Scene,
        instance_buffer: &mut InstanceBuffer,
        target: &metal::TextureRef,
        viewport_size: Size<DevicePixels>,
    ) -> Result<metal::CommandBuffer> {
        if scene.backdrops.is_empty() {
            self.backdrop_pool.clear();
            self.clear_blur_kernels();
        }
        let command_queue = self.command_queue.clone();
        let command_buffer = command_queue.new_command_buffer();
        let alpha = if self.layer.is_opaque() { 1. } else { 0. };
        let mut instance_offset = 0;

        let mut command_encoder =
            new_command_encoder(command_buffer, target, viewport_size, |color_attachment| {
                color_attachment.set_load_action(metal::MTLLoadAction::Clear);
                color_attachment.set_clear_color(metal::MTLClearColor::new(0., 0., 0., alpha));
            });

        for batch in scene.batches() {
            let ok = match batch {
                PrimitiveBatch::Backdrops(backdrops) => {
                    command_encoder = self.draw_backdrops(
                        backdrops,
                        command_buffer,
                        target,
                        viewport_size,
                        command_encoder,
                    );
                    true
                }
                PrimitiveBatch::Shadows(shadows) => self.draw_shadows(
                    shadows,
                    instance_buffer,
                    &mut instance_offset,
                    viewport_size,
                    command_encoder,
                ),
                PrimitiveBatch::Quads(quads) => self.draw_quads(
                    quads,
                    instance_buffer,
                    &mut instance_offset,
                    viewport_size,
                    command_encoder,
                ),
                PrimitiveBatch::Paths(paths) => {
                    command_encoder.end_encoding();

                    let did_draw = self.draw_paths_to_intermediate(
                        paths,
                        instance_buffer,
                        &mut instance_offset,
                        viewport_size,
                        command_buffer,
                    );

                    command_encoder = load_encoder(command_buffer, target, viewport_size);

                    if did_draw {
                        self.draw_paths_from_intermediate(
                            paths,
                            instance_buffer,
                            &mut instance_offset,
                            viewport_size,
                            command_encoder,
                        )
                    } else {
                        false
                    }
                }
                PrimitiveBatch::Underlines(underlines) => self.draw_underlines(
                    underlines,
                    instance_buffer,
                    &mut instance_offset,
                    viewport_size,
                    command_encoder,
                ),
                PrimitiveBatch::MonochromeSprites {
                    texture_id,
                    sprites,
                } => self.draw_monochrome_sprites(
                    texture_id,
                    sprites,
                    instance_buffer,
                    &mut instance_offset,
                    viewport_size,
                    command_encoder,
                ),
                PrimitiveBatch::PolychromeSprites {
                    texture_id,
                    sprites,
                } => self.draw_polychrome_sprites(
                    texture_id,
                    sprites,
                    instance_buffer,
                    &mut instance_offset,
                    viewport_size,
                    command_encoder,
                ),
                PrimitiveBatch::Surfaces(surfaces) => self.draw_surfaces(
                    surfaces,
                    instance_buffer,
                    &mut instance_offset,
                    viewport_size,
                    command_encoder,
                ),
            };
            if !ok {
                command_encoder.end_encoding();
                self.finish_backdrop_frame();
                anyhow::bail!(
                    "scene too large: {} paths, {} shadows, {} quads, {} underlines, {} mono, {} poly, {} surfaces, {} backdrops",
                    scene.paths.len(),
                    scene.shadows.len(),
                    scene.quads.len(),
                    scene.underlines.len(),
                    scene.monochrome_sprites.len(),
                    scene.polychrome_sprites.len(),
                    scene.surfaces.len(),
                    scene.backdrops.len(),
                );
            }
        }

        command_encoder.end_encoding();

        instance_buffer.metal_buffer.did_modify_range(NSRange {
            location: 0,
            length: instance_offset as NSUInteger,
        });
        self.finish_backdrop_frame();
        Ok(command_buffer.to_owned())
    }

    fn draw_backdrops<'a>(
        &mut self,
        backdrops: &[Backdrop],
        command_buffer: &'a metal::CommandBufferRef,
        target: &'a metal::TextureRef,
        viewport_size: Size<DevicePixels>,
        mut command_encoder: &'a metal::RenderCommandEncoderRef,
    ) -> &'a metal::RenderCommandEncoderRef {
        for backdrop in backdrops {
            match backdrop.phase {
                BackdropPhase::Capture => {
                    command_encoder.end_encoding();
                    self.capture_backdrop(backdrop, target, viewport_size, command_buffer);
                    command_encoder = load_encoder(command_buffer, target, viewport_size);
                }
                BackdropPhase::Composite => {
                    self.composite_backdrop(backdrop, viewport_size, command_encoder);
                }
            }
        }
        command_encoder
    }

    fn clear_blur_kernels(&mut self) {
        self.blur_kernels.clear();
    }

    fn finish_backdrop_frame(&mut self) {
        self.backdrop_pool.clear();
        // Reuse across in-flight frames is safe: textures are hazard-tracked by default,
        // and all work goes through one command queue.
        for capture in self.backdrop_frame.drain(..).flatten() {
            let blurred_is_sample = capture.blurred.as_ptr() == capture.sample.as_ptr();
            self.backdrop_pool.push(capture.sample);
            if !blurred_is_sample {
                self.backdrop_pool.push(capture.blurred);
            }
        }
    }

    fn backdrop_texture(&mut self, width: u64, height: u64) -> metal::Texture {
        // Exact sizes put the MPS edge at the captured region, never at stale pooled pixels.
        if let Some(index) = self
            .backdrop_pool
            .iter()
            .position(|texture| texture.width() == width && texture.height() == height)
        {
            return self.backdrop_pool.swap_remove(index);
        }
        let descriptor = metal::TextureDescriptor::new();
        descriptor.set_width(width);
        descriptor.set_height(height);
        descriptor.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        descriptor.set_storage_mode(metal::MTLStorageMode::Private);
        descriptor
            .set_usage(metal::MTLTextureUsage::ShaderRead | metal::MTLTextureUsage::ShaderWrite);
        self.device.new_texture(&descriptor)
    }

    fn blur_kernel(&mut self, sigma: f32) -> Option<NonNull<objc::runtime::Object>> {
        if !self.blur_supported {
            if !self.blur_unsupported_logged {
                log::error!(
                    "MPS Gaussian blur unsupported on this device; using backdrop snapshot"
                );
                self.blur_unsupported_logged = true;
            }
            return None;
        }
        let key = sigma.to_bits();
        if let Some(kernel) = self.blur_kernels.get(&key) {
            return Some(kernel.0);
        }
        let blur_class = objc::runtime::Class::get("MPSImageGaussianBlur")?;
        // MPS retains its device; BlurKernel owns the initialized kernel.
        let kernel = unsafe {
            let allocated: *mut objc::runtime::Object = msg_send![blur_class, alloc];
            let initialized: *mut objc::runtime::Object =
                msg_send![allocated, initWithDevice: self.device.as_ptr() sigma: sigma];
            let Some(kernel) = NonNull::new(initialized) else {
                if self.blur_initialization_failures.insert(key) {
                    log::error!(
                        "MPS Gaussian blur initialization failed for sigma {sigma}; using backdrop snapshot"
                    );
                }
                return None;
            };
            let _: () = msg_send![kernel.as_ptr(), setEdgeMode: self.blur_edge_mode];
            kernel
        };
        self.blur_kernels.insert(key, BlurKernel(kernel));
        Some(kernel)
    }

    fn capture_backdrop(
        &mut self,
        backdrop: &Backdrop,
        target: &metal::TextureRef,
        viewport_size: Size<DevicePixels>,
        command_buffer: &metal::CommandBufferRef,
    ) {
        let capture_index = backdrop.id as usize;
        if self.backdrop_frame.len() <= capture_index {
            self.backdrop_frame.resize_with(capture_index + 1, || None);
        }
        self.backdrop_frame[capture_index] = None;
        if target.framebuffer_only() {
            return;
        }
        let Some((origin, width, height)) =
            capture_region(backdrop.bounds, &backdrop.content_mask, viewport_size)
        else {
            return;
        };
        let sample = self.backdrop_texture(width, height);
        copy_region(command_buffer, target, &sample, origin, width, height);
        let blurred = if let Some(kernel) = self.blur_kernel(backdrop.sigma.0) {
            let blurred = self.backdrop_texture(width, height);
            encode_blur(kernel, command_buffer, &sample, &blurred);
            blurred
        } else {
            sample.clone()
        };
        self.backdrop_frame[capture_index] = Some(BackdropCapture {
            blurred,
            sample,
            origin,
        });
    }

    fn composite_backdrop(
        &self,
        backdrop: &Backdrop,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) {
        let Some(Some(capture)) = self.backdrop_frame.get(backdrop.id as usize) else {
            if backdrop.id == u32::MAX && !self.unmatched_composite_logged.replace(true) {
                log::error!("backdrop composite has no matching capture id");
            }
            debug_assert_ne!(
                backdrop.id,
                u32::MAX,
                "backdrop composite has no capture id"
            );
            return;
        };
        let uniforms = BackdropUniforms {
            bounds: backdrop.bounds,
            clip_bounds: backdrop.bounds.intersect(&backdrop.content_mask.bounds),
            corner_radii: backdrop.corner_radii,
            sample_origin: point(capture.origin.x.0 as f32, capture.origin.y.0 as f32),
            saturation: backdrop.saturation,
            opacity: backdrop.opacity,
            tint_alpha: backdrop.tint_alpha,
            padding: [0.; 3],
        };
        command_encoder.set_render_pipeline_state(&self.backdrop_composite_pipeline_state);
        command_encoder.set_vertex_buffer(0, Some(&self.unit_vertices), 0);
        command_encoder.set_vertex_bytes(
            1,
            mem::size_of_val(&uniforms) as u64,
            &uniforms as *const _ as *const c_void,
        );
        command_encoder.set_vertex_bytes(
            2,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const _ as *const c_void,
        );
        command_encoder.set_fragment_bytes(
            1,
            mem::size_of_val(&uniforms) as u64,
            &uniforms as *const _ as *const c_void,
        );
        command_encoder.set_fragment_texture(0, Some(&capture.blurred));
        command_encoder.set_fragment_texture(1, Some(&capture.sample));
        command_encoder.draw_primitives(metal::MTLPrimitiveType::Triangle, 0, 6);
    }

    fn draw_paths_to_intermediate(
        &self,
        paths: &[Path<ScaledPixels>],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_buffer: &metal::CommandBufferRef,
    ) -> bool {
        if paths.is_empty() {
            return true;
        }
        let Some(intermediate_texture) = &self.path_intermediate_texture else {
            return false;
        };

        let render_pass_descriptor = metal::RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_load_action(metal::MTLLoadAction::Clear);
        color_attachment.set_clear_color(metal::MTLClearColor::new(0., 0., 0., 0.));

        if let Some(msaa_texture) = &self.path_intermediate_msaa_texture {
            color_attachment.set_texture(Some(msaa_texture));
            color_attachment.set_resolve_texture(Some(intermediate_texture));
            color_attachment.set_store_action(metal::MTLStoreAction::MultisampleResolve);
        } else {
            color_attachment.set_texture(Some(intermediate_texture));
            color_attachment.set_store_action(metal::MTLStoreAction::Store);
        }

        let command_encoder = command_buffer.new_render_command_encoder(render_pass_descriptor);
        command_encoder.set_render_pipeline_state(&self.paths_rasterization_pipeline_state);

        align_offset(instance_offset);
        let mut vertices = Vec::new();
        for path in paths {
            vertices.extend(path.vertices.iter().map(|v| PathRasterizationVertex {
                xy_position: v.xy_position,
                st_position: v.st_position,
                color: path.color,
                bounds: path.bounds.intersect(&path.content_mask.bounds),
            }));
        }
        let vertices_bytes_len = mem::size_of_val(vertices.as_slice());
        let next_offset = *instance_offset + vertices_bytes_len;
        if next_offset > instance_buffer.size {
            command_encoder.end_encoding();
            return false;
        }
        command_encoder.set_vertex_buffer(
            PathRasterizationInputIndex::Vertices as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_vertex_bytes(
            PathRasterizationInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );
        command_encoder.set_fragment_buffer(
            PathRasterizationInputIndex::Vertices as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };
        unsafe {
            ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                buffer_contents,
                vertices_bytes_len,
            );
        }
        command_encoder.draw_primitives(
            metal::MTLPrimitiveType::Triangle,
            0,
            vertices.len() as u64,
        );
        *instance_offset = next_offset;

        command_encoder.end_encoding();
        true
    }

    fn draw_shadows(
        &self,
        shadows: &[Shadow],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        if shadows.is_empty() {
            return true;
        }
        align_offset(instance_offset);

        command_encoder.set_render_pipeline_state(&self.shadows_pipeline_state);
        command_encoder.set_vertex_buffer(
            ShadowInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_buffer(
            ShadowInputIndex::Shadows as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_fragment_buffer(
            ShadowInputIndex::Shadows as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );

        command_encoder.set_vertex_bytes(
            ShadowInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );

        let shadow_bytes_len = mem::size_of_val(shadows);
        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };

        let next_offset = *instance_offset + shadow_bytes_len;
        if next_offset > instance_buffer.size {
            return false;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                shadows.as_ptr() as *const u8,
                buffer_contents,
                shadow_bytes_len,
            );
        }

        command_encoder.draw_primitives_instanced(
            metal::MTLPrimitiveType::Triangle,
            0,
            6,
            shadows.len() as u64,
        );
        *instance_offset = next_offset;
        true
    }

    fn draw_quads(
        &self,
        quads: &[Quad],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        if quads.is_empty() {
            return true;
        }
        align_offset(instance_offset);

        command_encoder.set_render_pipeline_state(&self.quads_pipeline_state);
        command_encoder.set_vertex_buffer(
            QuadInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_buffer(
            QuadInputIndex::Quads as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_fragment_buffer(
            QuadInputIndex::Quads as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );

        command_encoder.set_vertex_bytes(
            QuadInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );

        let quad_bytes_len = mem::size_of_val(quads);
        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };

        let next_offset = *instance_offset + quad_bytes_len;
        if next_offset > instance_buffer.size {
            return false;
        }

        unsafe {
            ptr::copy_nonoverlapping(quads.as_ptr() as *const u8, buffer_contents, quad_bytes_len);
        }

        command_encoder.draw_primitives_instanced(
            metal::MTLPrimitiveType::Triangle,
            0,
            6,
            quads.len() as u64,
        );
        *instance_offset = next_offset;
        true
    }

    fn draw_paths_from_intermediate(
        &self,
        paths: &[Path<ScaledPixels>],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        let Some(first_path) = paths.first() else {
            return true;
        };

        let Some(ref intermediate_texture) = self.path_intermediate_texture else {
            return false;
        };

        command_encoder.set_render_pipeline_state(&self.path_sprites_pipeline_state);
        command_encoder.set_vertex_buffer(
            SpriteInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_bytes(
            SpriteInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );

        command_encoder.set_fragment_texture(
            SpriteInputIndex::AtlasTexture as u64,
            Some(intermediate_texture),
        );

        // When copying paths from the intermediate texture to the drawable,
        // each pixel must only be copied once, in case of transparent paths.
        //
        // If all paths have the same draw order, then their bounds are all
        // disjoint, so we can copy each path's bounds individually. If this
        // batch combines different draw orders, we perform a single copy
        // for a minimal spanning rect.
        let sprites;
        if paths.last().unwrap().order == first_path.order {
            sprites = paths
                .iter()
                .map(|path| PathSprite {
                    bounds: path.clipped_bounds(),
                })
                .collect();
        } else {
            let mut bounds = first_path.clipped_bounds();
            for path in paths.iter().skip(1) {
                bounds = bounds.union(&path.clipped_bounds());
            }
            sprites = vec![PathSprite { bounds }];
        }

        align_offset(instance_offset);
        let sprite_bytes_len = mem::size_of_val(sprites.as_slice());
        let next_offset = *instance_offset + sprite_bytes_len;
        if next_offset > instance_buffer.size {
            return false;
        }

        command_encoder.set_vertex_buffer(
            SpriteInputIndex::Sprites as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );

        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };
        unsafe {
            ptr::copy_nonoverlapping(
                sprites.as_ptr() as *const u8,
                buffer_contents,
                sprite_bytes_len,
            );
        }

        command_encoder.draw_primitives_instanced(
            metal::MTLPrimitiveType::Triangle,
            0,
            6,
            sprites.len() as u64,
        );
        *instance_offset = next_offset;

        true
    }

    fn draw_underlines(
        &self,
        underlines: &[Underline],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        if underlines.is_empty() {
            return true;
        }
        align_offset(instance_offset);

        command_encoder.set_render_pipeline_state(&self.underlines_pipeline_state);
        command_encoder.set_vertex_buffer(
            UnderlineInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_buffer(
            UnderlineInputIndex::Underlines as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_fragment_buffer(
            UnderlineInputIndex::Underlines as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );

        command_encoder.set_vertex_bytes(
            UnderlineInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );

        let underline_bytes_len = mem::size_of_val(underlines);
        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };

        let next_offset = *instance_offset + underline_bytes_len;
        if next_offset > instance_buffer.size {
            return false;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                underlines.as_ptr() as *const u8,
                buffer_contents,
                underline_bytes_len,
            );
        }

        command_encoder.draw_primitives_instanced(
            metal::MTLPrimitiveType::Triangle,
            0,
            6,
            underlines.len() as u64,
        );
        *instance_offset = next_offset;
        true
    }

    fn draw_monochrome_sprites(
        &self,
        texture_id: AtlasTextureId,
        sprites: &[MonochromeSprite],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        if sprites.is_empty() {
            return true;
        }
        align_offset(instance_offset);

        let sprite_bytes_len = mem::size_of_val(sprites);
        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };

        let next_offset = *instance_offset + sprite_bytes_len;
        if next_offset > instance_buffer.size {
            return false;
        }

        let texture = self.sprite_atlas.metal_texture(texture_id);
        let texture_size = size(
            DevicePixels(texture.width() as i32),
            DevicePixels(texture.height() as i32),
        );
        command_encoder.set_render_pipeline_state(&self.monochrome_sprites_pipeline_state);
        command_encoder.set_vertex_buffer(
            SpriteInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_buffer(
            SpriteInputIndex::Sprites as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_vertex_bytes(
            SpriteInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );
        command_encoder.set_vertex_bytes(
            SpriteInputIndex::AtlasTextureSize as u64,
            mem::size_of_val(&texture_size) as u64,
            &texture_size as *const Size<DevicePixels> as *const _,
        );
        command_encoder.set_fragment_buffer(
            SpriteInputIndex::Sprites as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_fragment_texture(SpriteInputIndex::AtlasTexture as u64, Some(&texture));

        unsafe {
            ptr::copy_nonoverlapping(
                sprites.as_ptr() as *const u8,
                buffer_contents,
                sprite_bytes_len,
            );
        }

        command_encoder.draw_primitives_instanced(
            metal::MTLPrimitiveType::Triangle,
            0,
            6,
            sprites.len() as u64,
        );
        *instance_offset = next_offset;
        true
    }

    fn draw_polychrome_sprites(
        &self,
        texture_id: AtlasTextureId,
        sprites: &[PolychromeSprite],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        if sprites.is_empty() {
            return true;
        }
        align_offset(instance_offset);

        let texture = self.sprite_atlas.metal_texture(texture_id);
        let texture_size = size(
            DevicePixels(texture.width() as i32),
            DevicePixels(texture.height() as i32),
        );
        command_encoder.set_render_pipeline_state(&self.polychrome_sprites_pipeline_state);
        command_encoder.set_vertex_buffer(
            SpriteInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_buffer(
            SpriteInputIndex::Sprites as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_vertex_bytes(
            SpriteInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );
        command_encoder.set_vertex_bytes(
            SpriteInputIndex::AtlasTextureSize as u64,
            mem::size_of_val(&texture_size) as u64,
            &texture_size as *const Size<DevicePixels> as *const _,
        );
        command_encoder.set_fragment_buffer(
            SpriteInputIndex::Sprites as u64,
            Some(&instance_buffer.metal_buffer),
            *instance_offset as u64,
        );
        command_encoder.set_fragment_texture(SpriteInputIndex::AtlasTexture as u64, Some(&texture));

        let sprite_bytes_len = mem::size_of_val(sprites);
        let buffer_contents =
            unsafe { (instance_buffer.metal_buffer.contents() as *mut u8).add(*instance_offset) };

        let next_offset = *instance_offset + sprite_bytes_len;
        if next_offset > instance_buffer.size {
            return false;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                sprites.as_ptr() as *const u8,
                buffer_contents,
                sprite_bytes_len,
            );
        }

        command_encoder.draw_primitives_instanced(
            metal::MTLPrimitiveType::Triangle,
            0,
            6,
            sprites.len() as u64,
        );
        *instance_offset = next_offset;
        true
    }

    fn draw_surfaces(
        &mut self,
        surfaces: &[PaintSurface],
        instance_buffer: &mut InstanceBuffer,
        instance_offset: &mut usize,
        viewport_size: Size<DevicePixels>,
        command_encoder: &metal::RenderCommandEncoderRef,
    ) -> bool {
        command_encoder.set_render_pipeline_state(&self.surfaces_pipeline_state);
        command_encoder.set_vertex_buffer(
            SurfaceInputIndex::Vertices as u64,
            Some(&self.unit_vertices),
            0,
        );
        command_encoder.set_vertex_bytes(
            SurfaceInputIndex::ViewportSize as u64,
            mem::size_of_val(&viewport_size) as u64,
            &viewport_size as *const Size<DevicePixels> as *const _,
        );

        for surface in surfaces {
            let texture_size = size(
                DevicePixels::from(surface.image_buffer.get_width() as i32),
                DevicePixels::from(surface.image_buffer.get_height() as i32),
            );

            assert_eq!(
                surface.image_buffer.get_pixel_format(),
                kCVPixelFormatType_420YpCbCr8BiPlanarFullRange
            );

            let y_texture = self
                .core_video_texture_cache
                .create_texture_from_image(
                    surface.image_buffer.as_concrete_TypeRef(),
                    None,
                    MTLPixelFormat::R8Unorm,
                    surface.image_buffer.get_width_of_plane(0),
                    surface.image_buffer.get_height_of_plane(0),
                    0,
                )
                .unwrap();
            let cb_cr_texture = self
                .core_video_texture_cache
                .create_texture_from_image(
                    surface.image_buffer.as_concrete_TypeRef(),
                    None,
                    MTLPixelFormat::RG8Unorm,
                    surface.image_buffer.get_width_of_plane(1),
                    surface.image_buffer.get_height_of_plane(1),
                    1,
                )
                .unwrap();

            align_offset(instance_offset);
            let next_offset = *instance_offset + mem::size_of::<Surface>();
            if next_offset > instance_buffer.size {
                return false;
            }

            command_encoder.set_vertex_buffer(
                SurfaceInputIndex::Surfaces as u64,
                Some(&instance_buffer.metal_buffer),
                *instance_offset as u64,
            );
            command_encoder.set_vertex_bytes(
                SurfaceInputIndex::TextureSize as u64,
                mem::size_of_val(&texture_size) as u64,
                &texture_size as *const Size<DevicePixels> as *const _,
            );
            // let y_texture = y_texture.get_texture().unwrap().
            command_encoder.set_fragment_texture(SurfaceInputIndex::YTexture as u64, unsafe {
                let texture = CVMetalTextureGetTexture(y_texture.as_concrete_TypeRef());
                Some(metal::TextureRef::from_ptr(texture as *mut _))
            });
            command_encoder.set_fragment_texture(SurfaceInputIndex::CbCrTexture as u64, unsafe {
                let texture = CVMetalTextureGetTexture(cb_cr_texture.as_concrete_TypeRef());
                Some(metal::TextureRef::from_ptr(texture as *mut _))
            });

            unsafe {
                let buffer_contents = (instance_buffer.metal_buffer.contents() as *mut u8)
                    .add(*instance_offset)
                    as *mut SurfaceBounds;
                ptr::write(
                    buffer_contents,
                    SurfaceBounds {
                        bounds: surface.bounds,
                        content_mask: surface.content_mask.clone(),
                    },
                );
            }

            command_encoder.draw_primitives(metal::MTLPrimitiveType::Triangle, 0, 6);
            *instance_offset = next_offset;
        }
        true
    }
}

fn capture_region(
    bounds: Bounds<ScaledPixels>,
    content_mask: &ContentMask<ScaledPixels>,
    viewport_size: Size<DevicePixels>,
) -> Option<(Point<DevicePixels>, u64, u64)> {
    let bounds = bounds
        .intersect(&content_mask.bounds)
        .intersect(&Bounds::new(
            Point::default(),
            viewport_size.map(|dimension| ScaledPixels(dimension.0 as f32)),
        ));
    if bounds.is_empty() {
        return None;
    }
    let origin = point(
        DevicePixels(bounds.origin.x.0.floor() as i32),
        DevicePixels(bounds.origin.y.0.floor() as i32),
    );
    let width = (bounds.right().0.ceil() as i32 - origin.x.0) as u64;
    let height = (bounds.bottom().0.ceil() as i32 - origin.y.0) as u64;
    Some((origin, width, height))
}

fn load_encoder<'a>(
    command_buffer: &'a metal::CommandBufferRef,
    target: &'a metal::TextureRef,
    viewport_size: Size<DevicePixels>,
) -> &'a metal::RenderCommandEncoderRef {
    new_command_encoder(command_buffer, target, viewport_size, |attachment| {
        attachment.set_load_action(metal::MTLLoadAction::Load);
    })
}

fn new_command_encoder<'a>(
    command_buffer: &'a metal::CommandBufferRef,
    target: &'a metal::TextureRef,
    viewport_size: Size<DevicePixels>,
    configure_color_attachment: impl Fn(&RenderPassColorAttachmentDescriptorRef),
) -> &'a metal::RenderCommandEncoderRef {
    let render_pass_descriptor = metal::RenderPassDescriptor::new();
    let color_attachment = render_pass_descriptor
        .color_attachments()
        .object_at(0)
        .unwrap();
    color_attachment.set_texture(Some(target));
    color_attachment.set_store_action(metal::MTLStoreAction::Store);
    configure_color_attachment(color_attachment);

    let command_encoder = command_buffer.new_render_command_encoder(render_pass_descriptor);
    command_encoder.set_viewport(metal::MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: i32::from(viewport_size.width) as f64,
        height: i32::from(viewport_size.height) as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    command_encoder
}

fn build_pipeline_state(
    device: &metal::DeviceRef,
    library: &metal::LibraryRef,
    label: &str,
    vertex_fn_name: &str,
    fragment_fn_name: &str,
    pixel_format: metal::MTLPixelFormat,
) -> metal::RenderPipelineState {
    let descriptor = pipeline_descriptor(
        library,
        label,
        vertex_fn_name,
        fragment_fn_name,
        pixel_format,
    );
    let color_attachment = descriptor.color_attachments().object_at(0).unwrap();
    color_attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
    color_attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::One);
    color_attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
    color_attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::One);

    device
        .new_render_pipeline_state(&descriptor)
        .expect("could not create render pipeline state")
}

fn pipeline_descriptor(
    library: &metal::LibraryRef,
    label: &str,
    vertex_fn_name: &str,
    fragment_fn_name: &str,
    pixel_format: metal::MTLPixelFormat,
) -> metal::RenderPipelineDescriptor {
    let vertex_fn = library
        .get_function(vertex_fn_name, None)
        .expect("error locating vertex function");
    let fragment_fn = library
        .get_function(fragment_fn_name, None)
        .expect("error locating fragment function");

    let descriptor = metal::RenderPipelineDescriptor::new();
    descriptor.set_label(label);
    descriptor.set_vertex_function(Some(vertex_fn.as_ref()));
    descriptor.set_fragment_function(Some(fragment_fn.as_ref()));
    let color_attachment = descriptor.color_attachments().object_at(0).unwrap();
    color_attachment.set_pixel_format(pixel_format);
    color_attachment.set_blending_enabled(true);
    color_attachment.set_rgb_blend_operation(metal::MTLBlendOperation::Add);
    color_attachment.set_alpha_blend_operation(metal::MTLBlendOperation::Add);
    descriptor
}

fn build_backdrop_pipeline_state(
    device: &metal::DeviceRef,
    library: &metal::LibraryRef,
) -> metal::RenderPipelineState {
    let descriptor = pipeline_descriptor(
        library,
        "backdrop_composite",
        "backdrop_composite_vertex",
        "backdrop_composite_fragment",
        MTLPixelFormat::BGRA8Unorm,
    );
    let attachment = descriptor.color_attachments().object_at(0).unwrap();
    attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::Source1Alpha);
    attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::Source1Alpha);
    attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSource1Alpha);
    attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSource1Alpha);
    device
        .new_render_pipeline_state(&descriptor)
        .expect("could not create backdrop composite pipeline")
}

fn build_path_sprite_pipeline_state(
    device: &metal::DeviceRef,
    library: &metal::LibraryRef,
    label: &str,
    vertex_fn_name: &str,
    fragment_fn_name: &str,
    pixel_format: metal::MTLPixelFormat,
) -> metal::RenderPipelineState {
    let vertex_fn = library
        .get_function(vertex_fn_name, None)
        .expect("error locating vertex function");
    let fragment_fn = library
        .get_function(fragment_fn_name, None)
        .expect("error locating fragment function");

    let descriptor = metal::RenderPipelineDescriptor::new();
    descriptor.set_label(label);
    descriptor.set_vertex_function(Some(vertex_fn.as_ref()));
    descriptor.set_fragment_function(Some(fragment_fn.as_ref()));
    let color_attachment = descriptor.color_attachments().object_at(0).unwrap();
    color_attachment.set_pixel_format(pixel_format);
    color_attachment.set_blending_enabled(true);
    color_attachment.set_rgb_blend_operation(metal::MTLBlendOperation::Add);
    color_attachment.set_alpha_blend_operation(metal::MTLBlendOperation::Add);
    color_attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::One);
    color_attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::One);
    color_attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
    color_attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::One);

    device
        .new_render_pipeline_state(&descriptor)
        .expect("could not create render pipeline state")
}

fn build_path_rasterization_pipeline_state(
    device: &metal::DeviceRef,
    library: &metal::LibraryRef,
    label: &str,
    vertex_fn_name: &str,
    fragment_fn_name: &str,
    pixel_format: metal::MTLPixelFormat,
    path_sample_count: u32,
) -> metal::RenderPipelineState {
    let vertex_fn = library
        .get_function(vertex_fn_name, None)
        .expect("error locating vertex function");
    let fragment_fn = library
        .get_function(fragment_fn_name, None)
        .expect("error locating fragment function");

    let descriptor = metal::RenderPipelineDescriptor::new();
    descriptor.set_label(label);
    descriptor.set_vertex_function(Some(vertex_fn.as_ref()));
    descriptor.set_fragment_function(Some(fragment_fn.as_ref()));
    if path_sample_count > 1 {
        descriptor.set_raster_sample_count(path_sample_count as _);
        descriptor.set_alpha_to_coverage_enabled(false);
    }
    let color_attachment = descriptor.color_attachments().object_at(0).unwrap();
    color_attachment.set_pixel_format(pixel_format);
    color_attachment.set_blending_enabled(true);
    color_attachment.set_rgb_blend_operation(metal::MTLBlendOperation::Add);
    color_attachment.set_alpha_blend_operation(metal::MTLBlendOperation::Add);
    color_attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::One);
    color_attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::One);
    color_attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
    color_attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

    device
        .new_render_pipeline_state(&descriptor)
        .expect("could not create render pipeline state")
}

// Align to multiples of 256 make Metal happy.
fn align_offset(offset: &mut usize) {
    *offset = (*offset).div_ceil(256) * 256;
}

#[repr(C)]
enum ShadowInputIndex {
    Vertices = 0,
    Shadows = 1,
    ViewportSize = 2,
}

#[repr(C)]
enum QuadInputIndex {
    Vertices = 0,
    Quads = 1,
    ViewportSize = 2,
}

#[repr(C)]
enum UnderlineInputIndex {
    Vertices = 0,
    Underlines = 1,
    ViewportSize = 2,
}

#[repr(C)]
enum SpriteInputIndex {
    Vertices = 0,
    Sprites = 1,
    ViewportSize = 2,
    AtlasTextureSize = 3,
    AtlasTexture = 4,
}

#[repr(C)]
enum SurfaceInputIndex {
    Vertices = 0,
    Surfaces = 1,
    ViewportSize = 2,
    TextureSize = 3,
    YTexture = 4,
    CbCrTexture = 5,
}

#[repr(C)]
enum PathRasterizationInputIndex {
    Vertices = 0,
    ViewportSize = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct PathSprite {
    pub bounds: Bounds<ScaledPixels>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct SurfaceBounds {
    pub bounds: Bounds<ScaledPixels>,
    pub content_mask: ContentMask<ScaledPixels>,
}

fn copy_region(
    command_buffer: &metal::CommandBufferRef,
    target: &metal::TextureRef,
    destination: &metal::TextureRef,
    origin: Point<DevicePixels>,
    width: u64,
    height: u64,
) {
    let blit = command_buffer.new_blit_command_encoder();
    blit.copy_from_texture(
        target,
        0,
        0,
        metal::MTLOrigin {
            x: origin.x.0 as u64,
            y: origin.y.0 as u64,
            z: 0,
        },
        metal::MTLSize {
            width,
            height,
            depth: 1,
        },
        destination,
        0,
        0,
        metal::MTLOrigin { x: 0, y: 0, z: 0 },
    );
    blit.end_encoding();
}

fn encode_blur(
    kernel: NonNull<objc::runtime::Object>,
    command_buffer: &metal::CommandBufferRef,
    source: &metal::TextureRef,
    destination: &metal::TextureRef,
) {
    // The encoder has ended and source and destination are distinct retained textures.
    unsafe {
        let _: () = msg_send![kernel.as_ptr(), encodeToCommandBuffer: command_buffer.as_ptr() sourceTexture: source.as_ptr() destinationTexture: destination.as_ptr()];
    }
}

#[cfg(test)]
mod backdrop_tests {
    use super::*;
    use crate::{Backdrop, Corners, black, rgb};

    const TEST_SIZE: i32 = 200;
    const STRIPE_WIDTH: i32 = 4;
    const GLASS_LEFT: i32 = 50;
    const GLASS_TOP: i32 = 50;
    const GLASS_WIDTH: i32 = 100;
    const GLASS_HEIGHT: i32 = 100;
    const GLASS_CORNER_RADIUS: f32 = 20.;
    const GLASS_ANTI_ALIASING_BAND: f32 = 2.;

    fn is_inside_rounded_glass(column: i32, row: i32) -> bool {
        let horizontal_distance = (GLASS_LEFT as f32 + GLASS_CORNER_RADIUS - column as f32)
            .max(column as f32 - (GLASS_LEFT + GLASS_WIDTH) as f32 + GLASS_CORNER_RADIUS)
            .max(0.);
        let vertical_distance = (GLASS_TOP as f32 + GLASS_CORNER_RADIUS - row as f32)
            .max(row as f32 - (GLASS_TOP + GLASS_HEIGHT) as f32 + GLASS_CORNER_RADIUS)
            .max(0.);
        let interior_radius = GLASS_CORNER_RADIUS - GLASS_ANTI_ALIASING_BAND;
        horizontal_distance * horizontal_distance + vertical_distance * vertical_distance
            < interior_radius * interior_radius
    }

    fn viewport() -> Bounds<ScaledPixels> {
        Bounds::new(
            point(ScaledPixels(0.), ScaledPixels(0.)),
            size(
                ScaledPixels(TEST_SIZE as f32),
                ScaledPixels(TEST_SIZE as f32),
            ),
        )
    }

    #[derive(Clone)]
    struct GlassInput {
        bounds: Bounds<crate::Pixels>,
        radius: crate::Pixels,
        corner_radii: Corners<crate::Pixels>,
        content_mask: ContentMask<crate::Pixels>,
        saturation: f32,
        opacity: f32,
    }

    impl GlassInput {
        fn pair(self) -> Option<(Backdrop, Backdrop)> {
            Backdrop::pair(
                self.bounds,
                self.corner_radii,
                self.content_mask,
                crate::BackdropBlur {
                    radius: self.radius,
                    saturation: self.saturation,
                },
                1.,
                self.opacity,
            )
        }
    }

    fn glass(opacity: f32) -> GlassInput {
        GlassInput {
            bounds: Bounds::new(
                point(crate::px(GLASS_LEFT as f32), crate::px(GLASS_TOP as f32)),
                size(
                    crate::px(GLASS_WIDTH as f32),
                    crate::px(GLASS_HEIGHT as f32),
                ),
            ),
            radius: crate::px(8.),
            corner_radii: Corners::all(crate::px(GLASS_CORNER_RADIUS)),
            content_mask: ContentMask {
                bounds: viewport().map(|value| crate::px(value.0)),
            },
            saturation: 1.,
            opacity,
        }
    }

    fn stripe_scene() -> Scene {
        let mut scene = Scene::default();
        for stripe in 0..TEST_SIZE / STRIPE_WIDTH {
            let color = if stripe % 2 == 0 { 0x000000 } else { 0xffffff };
            scene.insert_primitive(Quad {
                bounds: Bounds::new(
                    point(
                        ScaledPixels((stripe * STRIPE_WIDTH) as f32),
                        ScaledPixels(0.),
                    ),
                    size(
                        ScaledPixels(STRIPE_WIDTH as f32),
                        ScaledPixels(TEST_SIZE as f32),
                    ),
                ),
                content_mask: ContentMask { bounds: viewport() },
                background: rgb(color).into(),
                ..Quad::default()
            });
        }
        scene
    }

    fn render_to_pixels(
        renderer: &mut MetalRenderer,
        scene: &mut Scene,
        size: Size<DevicePixels>,
    ) -> Vec<u8> {
        scene.finish();
        let descriptor = metal::TextureDescriptor::new();
        descriptor.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        descriptor.set_width(size.width.0 as u64);
        descriptor.set_height(size.height.0 as u64);
        descriptor.set_storage_mode(metal::MTLStorageMode::Managed);
        descriptor
            .set_usage(metal::MTLTextureUsage::RenderTarget | metal::MTLTextureUsage::ShaderRead);
        let texture = renderer.device.new_texture(&descriptor);
        let mut instance_buffer = renderer
            .instance_buffer_pool
            .lock()
            .acquire(&renderer.device);
        let command_buffer = renderer
            .draw_primitives(scene, &mut instance_buffer, &texture, size)
            .expect("offscreen scene must render");
        let blit = command_buffer.new_blit_command_encoder();
        blit.synchronize_resource(&texture);
        blit.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        assert_eq!(
            command_buffer.status(),
            metal::MTLCommandBufferStatus::Completed
        );
        renderer
            .instance_buffer_pool
            .lock()
            .release(instance_buffer);
        let bytes_per_row = size.width.0 as usize * 4;
        let mut pixels = vec![0; bytes_per_row * size.height.0 as usize];
        texture.get_bytes(
            pixels.as_mut_ptr().cast(),
            bytes_per_row as u64,
            metal::MTLRegion::new_2d(0, 0, size.width.0 as u64, size.height.0 as u64),
            0,
        );
        pixels
    }

    fn render(renderer: &mut MetalRenderer, scene: &mut Scene) -> Vec<u8> {
        render_to_pixels(
            renderer,
            scene,
            size(DevicePixels(TEST_SIZE), DevicePixels(TEST_SIZE)),
        )
    }

    fn pixel(pixels: &[u8], column: usize, row: usize) -> &[u8] {
        let offset = (row * TEST_SIZE as usize + column) * 4;
        &pixels[offset..offset + 4]
    }

    fn renderer() -> MetalRenderer {
        assert!(
            !metal::Device::all().is_empty(),
            "GPU pixel tests require an accessible Metal device"
        );
        MetalRenderer::new(Arc::new(Mutex::new(InstanceBufferPool::default())))
    }

    fn rectangle(left: f32, top: f32, width: f32, height: f32) -> Bounds<ScaledPixels> {
        Bounds::new(
            point(ScaledPixels(left), ScaledPixels(top)),
            size(ScaledPixels(width), ScaledPixels(height)),
        )
    }

    fn insert_glass(scene: &mut Scene, input: GlassInput) {
        let (capture, composite) = input.pair().expect("glass must produce a backdrop pair");
        scene.insert_primitive(capture);
        scene.insert_primitive(composite);
    }

    fn solid_scene(color: u32) -> Scene {
        let mut scene = Scene::default();
        scene.insert_primitive(Quad {
            bounds: viewport(),
            content_mask: ContentMask { bounds: viewport() },
            background: rgb(color).into(),
            ..Quad::default()
        });
        scene
    }

    fn shadow_scene(opacity: f32) -> Scene {
        let mut scene = solid_scene(0xffffff);
        let (capture, composite) = glass(opacity).pair().unwrap();
        scene.insert_primitive(capture);
        scene.insert_primitive(Shadow {
            order: 0,
            blur_radius: ScaledPixels(30.),
            bounds: composite.bounds,
            corner_radii: Corners::all(ScaledPixels(20.)),
            content_mask: ContentMask { bounds: viewport() },
            color: black().opacity(opacity),
        });
        scene.insert_primitive(composite);
        scene
    }

    fn assert_flat(pixels: &[u8], columns: std::ops::Range<usize>, row: usize) {
        for column in columns {
            assert!(
                pixel(pixels, column, row)[0].abs_diff(pixel(pixels, column + 1, row)[0]) < 10,
                "blurred columns {column} and {} differ too much",
                column + 1
            );
        }
    }

    fn assert_rgb(actual: &[u8], expected: [u8; 3], tolerance: u8) {
        assert!(
            actual[2].abs_diff(expected[0]) <= tolerance,
            "red: {actual:?}"
        );
        assert!(
            actual[1].abs_diff(expected[1]) <= tolerance,
            "green: {actual:?}"
        );
        assert!(
            actual[0].abs_diff(expected[2]) <= tolerance,
            "blue: {actual:?}"
        );
    }

    #[test]
    fn capture_region_clips_before_rounding_to_device_pixels() {
        let mask = ContentMask {
            bounds: rectangle(10.25, 20.75, 80., 90.),
        };
        let region = capture_region(
            rectangle(-5., 15., 70.5, 150.),
            &mask,
            size(DevicePixels(200), DevicePixels(100)),
        )
        .unwrap();
        assert_eq!(region, (point(DevicePixels(10), DevicePixels(20)), 56, 80));
        assert!(
            capture_region(
                rectangle(150., 150., 10., 10.),
                &mask,
                size(DevicePixels(TEST_SIZE), DevicePixels(TEST_SIZE)),
            )
            .is_none()
        );
    }

    #[test]
    fn blur_ignores_content_outside_border_box() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = solid_scene(0x000000);
            scene.insert_primitive(Quad {
                bounds: rectangle(44., 0., 6., TEST_SIZE as f32),
                content_mask: ContentMask { bounds: viewport() },
                background: rgb(0xffffff).into(),
                ..Quad::default()
            });
            insert_glass(
                &mut scene,
                GlassInput {
                    corner_radii: Corners::default(),
                    ..glass(1.)
                },
            );
            let pixels = render(&mut renderer, &mut scene);
            for row in 50..150 {
                for column in 50..60 {
                    assert!(pixel(&pixels, column, row)[0] < 6);
                }
            }
        });
    }

    #[test]
    fn blur_mirrors_at_border_box_edge() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = solid_scene(0x808080);
            insert_glass(
                &mut scene,
                GlassInput {
                    corner_radii: Corners::default(),
                    ..glass(1.)
                },
            );
            let pixels = render(&mut renderer, &mut scene);
            for coordinate in 50..150 {
                assert_rgb(pixel(&pixels, 50, coordinate), [128; 3], 2);
                assert_rgb(pixel(&pixels, 149, coordinate), [128; 3], 2);
                assert_rgb(pixel(&pixels, coordinate, 50), [128; 3], 2);
                assert_rgb(pixel(&pixels, coordinate, 149), [128; 3], 2);
            }
        });
    }

    #[test]
    fn half_opacity_glass_matches_css_group() {
        objc::rc::autoreleasepool(|| {
            const TINT_ALPHA: f32 = 0.86;
            const OPACITY: f32 = 0.5;
            let mut renderer = renderer();
            let mut scene = stripe_scene();
            let (capture, mut composite) = glass(OPACITY).pair().unwrap();
            composite.tint_alpha = TINT_ALPHA;
            let bounds = composite.bounds;
            scene.insert_primitive(capture);
            scene.insert_primitive(composite);
            let mut tint = crate::Hsla::from(crate::rgba(0x222224ff));
            tint.a = TINT_ALPHA * OPACITY;
            scene.insert_primitive(Quad {
                bounds,
                content_mask: ContentMask { bounds: viewport() },
                background: tint.into(),
                ..Quad::default()
            });
            let pixels = render(&mut renderer, &mut scene);
            for column in [100, 104] {
                let original = if (column as i32 / STRIPE_WIDTH) % 2 == 0 {
                    0.
                } else {
                    255.
                };
                let expected = OPACITY * (TINT_ALPHA * 34. + (1. - TINT_ALPHA) * 127.)
                    + (1. - OPACITY) * original;
                assert_rgb(pixel(&pixels, column, 100), [expected.round() as u8; 3], 4);
            }
        });
    }

    #[test]
    fn offset_shadow_stays_between_capture_and_composite() {
        let mut scene = solid_scene(0xffffff);
        let input = GlassInput {
            bounds: Bounds::new(
                point(crate::px(50.), crate::px(50.)),
                size(crate::px(100.), crate::px(40.)),
            ),
            ..glass(1.)
        };
        let shadows = [crate::BoxShadow {
            color: black(),
            offset: point(crate::px(0.), crate::px(60.)),
            blur_radius: crate::px(30.),
            spread_radius: crate::px(0.),
        }];
        let footprint = crate::style::shadow_footprint(input.bounds, &shadows).scale(1.);
        let (capture, composite) = input.pair().unwrap();
        scene.insert_primitive(capture);
        scene.push_layer(footprint);
        scene.insert_primitive(Shadow {
            order: 0,
            bounds: rectangle(50., 110., 100., 40.),
            blur_radius: ScaledPixels(30.),
            corner_radii: composite.corner_radii,
            content_mask: ContentMask { bounds: viewport() },
            color: black(),
        });
        scene.pop_layer();
        scene.insert_primitive(composite);
        let mut replay = Scene::default();
        replay.replay(0..scene.len(), &scene);
        for mut ordered_scene in [scene, replay] {
            ordered_scene.finish();
            assert!(ordered_scene.backdrops[0].order < ordered_scene.shadows[0].order);
            assert!(ordered_scene.shadows[0].order < ordered_scene.backdrops[1].order);
            objc::rc::autoreleasepool(|| {
                let pixels = render(&mut renderer(), &mut ordered_scene);
                for row in 60..80 {
                    for column in 70..130 {
                        assert!(pixel(&pixels, column, row)[0] >= 250);
                    }
                }
            });
        }
    }

    #[test]
    fn prepare_layer_follows_glass() {
        objc::rc::autoreleasepool(|| {
            let renderer = renderer();
            let mut scene = Scene::default();
            insert_glass(&mut scene, glass(1.));
            renderer.prepare_layer(&scene);
            assert!(!renderer.layer.framebuffer_only());
            renderer.prepare_layer(&Scene::default());
            assert!(renderer.layer.framebuffer_only());
        });
    }

    #[test]
    fn glass_without_blur_keeps_snapshots_separate() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            renderer.blur_supported = false;
            for _ in 0..2 {
                let mut scene = solid_scene(0x000000);
                scene.insert_primitive(Quad {
                    bounds: rectangle(100., 0., 100., 200.),
                    content_mask: ContentMask { bounds: viewport() },
                    background: rgb(0xffffff).into(),
                    ..Quad::default()
                });
                for left in [20., 120.] {
                    insert_glass(
                        &mut scene,
                        GlassInput {
                            bounds: Bounds::new(
                                point(crate::px(left), crate::px(60.)),
                                size(crate::px(60.), crate::px(80.)),
                            ),
                            ..glass(1.)
                        },
                    );
                }
                let pixels = render(&mut renderer, &mut scene);
                assert_rgb(pixel(&pixels, 50, 100), [0; 3], 2);
                assert_rgb(pixel(&pixels, 150, 100), [255; 3], 2);
                assert_eq!(renderer.backdrop_pool.len(), 2);
                assert_ne!(
                    renderer.backdrop_pool[0].as_ptr(),
                    renderer.backdrop_pool[1].as_ptr()
                );
            }
        });
    }

    #[test]
    fn glass_without_blur_still_hides_shadow() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            renderer.blur_supported = false;
            let original = render(&mut renderer, &mut stripe_scene());
            for _ in 0..2 {
                let mut scene = stripe_scene();
                let (capture, composite) = glass(1.).pair().unwrap();
                scene.insert_primitive(capture);
                scene.insert_primitive(Shadow {
                    order: 0,
                    blur_radius: ScaledPixels(30.),
                    bounds: composite.bounds,
                    corner_radii: Corners::all(ScaledPixels(GLASS_CORNER_RADIUS)),
                    content_mask: ContentMask { bounds: viewport() },
                    color: black(),
                });
                scene.insert_primitive(composite);
                let pixels = render(&mut renderer, &mut scene);
                for row in GLASS_TOP + 1..GLASS_TOP + GLASS_HEIGHT - 1 {
                    for column in GLASS_LEFT + 1..GLASS_LEFT + GLASS_WIDTH - 1 {
                        if is_inside_rounded_glass(column, row) {
                            let column = column as usize;
                            let row = row as usize;
                            let expected = pixel(&original, column, row)[0];
                            assert_rgb(pixel(&pixels, column, row), [expected; 3], 2);
                        }
                    }
                }
                assert!(pixel(&pixels, 100, 40)[0] < pixel(&original, 100, 40)[0]);
                assert_eq!(renderer.backdrop_pool.len(), 1);
                assert!(renderer.blur_kernels.is_empty());
            }
        });
    }

    #[test]
    fn backdrop_hides_own_shadow_at_half_opacity() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = shadow_scene(0.5);
            let pixels = render(&mut renderer, &mut scene);
            assert!(pixel(&pixels, 100, 100)[0] >= 250);
            assert!(pixel(&pixels, 40, 100)[0] < 245);
        });
    }

    #[test]
    fn backdrop_half_opacity_mixes_original_and_blur() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = stripe_scene();
            insert_glass(&mut scene, glass(0.5));
            let pixels = render(&mut renderer, &mut scene);
            let contrast = pixel(&pixels, 99, 100)[0].abs_diff(pixel(&pixels, 100, 100)[0]);
            assert!(
                (100..=155).contains(&contrast),
                "stripe contrast: {contrast}"
            );
        });
    }

    #[test]
    fn backdrop_blur_matches_expected_values() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = solid_scene(0x000000);
            scene.insert_primitive(Quad {
                bounds: rectangle(100., 0., 100., 200.),
                content_mask: ContentMask { bounds: viewport() },
                background: rgb(0xffffff).into(),
                ..Quad::default()
            });
            insert_glass(&mut scene, glass(1.));
            let pixels = render(&mut renderer, &mut scene);
            assert!((100..=155).contains(&pixel(&pixels, 100, 100)[0]));
            assert!(pixel(&pixels, 62, 100)[0] < 30);
            assert!(pixel(&pixels, 138, 100)[0] > 225);

            let mut stripes = stripe_scene();
            insert_glass(&mut stripes, glass(1.));
            assert_flat(&render(&mut renderer, &mut stripes), 52..60, 100);
        });
    }

    #[test]
    fn backdrop_saturation_follows_css_matrix() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            for saturation in [0., 1., 1.5] {
                let mut scene = solid_scene(0x806040);
                insert_glass(
                    &mut scene,
                    GlassInput {
                        radius: crate::px(4.),
                        saturation,
                        ..glass(1.)
                    },
                );
                let pixels = render(&mut renderer, &mut scene);
                let center = pixel(&pixels, 100, 100);
                if saturation == 0. {
                    assert!(center[0].abs_diff(center[1]) <= 2);
                    assert!(center[1].abs_diff(center[2]) <= 2);
                    assert!(center[0].abs_diff(center[2]) <= 2);
                    assert_rgb(center, [100, 100, 100], 1);
                } else if saturation == 1. {
                    assert_rgb(center, [128, 96, 64], 2);
                } else {
                    assert!(i16::from(center[2]) - i16::from(center[0]) >= 84);
                }
            }
        });
    }

    #[test]
    fn two_glass_elements_blur_independently() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = solid_scene(0x000000);
            for step_left in [40., 140.] {
                scene.insert_primitive(Quad {
                    bounds: rectangle(step_left, 0., 60., 200.),
                    content_mask: ContentMask { bounds: viewport() },
                    background: rgb(0xffffff).into(),
                    ..Quad::default()
                });
            }
            for (left, radius) in [(20., 4.), (120., 12.)] {
                insert_glass(
                    &mut scene,
                    GlassInput {
                        bounds: rectangle(left, 50., 60., 100.).map(|value| crate::px(value.0)),
                        radius: crate::px(radius),
                        ..glass(1.)
                    },
                );
            }
            let pixels = render(&mut renderer, &mut scene);
            assert!(pixel(&pixels, 34, 100)[0] < 25);
            assert!(pixel(&pixels, 134, 100)[0] > 60);
        });
    }

    #[test]
    fn backdrop_respects_viewport_and_mask_clipping() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let original = render(&mut renderer, &mut stripe_scene());
            let mut scene = stripe_scene();
            insert_glass(
                &mut scene,
                GlassInput {
                    bounds: rectangle(-30., 50., 100., 100.).map(|value| crate::px(value.0)),
                    ..glass(1.)
                },
            );
            let pixels = render(&mut renderer, &mut scene);
            assert_flat(&pixels, 10..60, 100);
            assert_rgb(pixel(&pixels, 30, 100), [127, 127, 127], 5);
            assert_eq!(pixel(&pixels, 75, 100), pixel(&original, 75, 100));

            let mut scene = stripe_scene();
            insert_glass(
                &mut scene,
                GlassInput {
                    content_mask: ContentMask {
                        bounds: rectangle(50., 50., 50., 100.).map(|value| crate::px(value.0)),
                    },
                    ..glass(1.)
                },
            );
            let pixels = render(&mut renderer, &mut scene);
            assert_flat(&pixels, 55..95, 100);
            assert_rgb(pixel(&pixels, 75, 100), [127, 127, 127], 5);
            for row in 50..150 {
                for column in 100..150 {
                    assert_eq!(pixel(&pixels, column, row), pixel(&original, column, row));
                }
            }
        });
    }

    #[test]
    fn scratch_pool_stays_bounded_while_glass_resizes() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            for width in [50., 60., 70., 80., 90.] {
                let mut scene = stripe_scene();
                insert_glass(
                    &mut scene,
                    GlassInput {
                        bounds: rectangle(50., 50., width, 100.).map(|value| crate::px(value.0)),
                        ..glass(1.)
                    },
                );
                render(&mut renderer, &mut scene);
                assert!(renderer.backdrop_pool.len() <= 2);
            }
        });
    }

    #[test]
    fn backdrop_blurs_only_inside_rounded_bounds() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let original = render(&mut renderer, &mut stripe_scene());
            let mut scene = stripe_scene();
            insert_glass(&mut scene, glass(1.));
            let pixels = render(&mut renderer, &mut scene);
            assert_flat(&pixels, 90..110, 100);
            assert_eq!(pixel(&pixels, 20, 100), pixel(&original, 20, 100));
            assert_eq!(pixel(&pixels, 52, 52), pixel(&original, 52, 52));
        });
    }

    #[test]
    fn backdrop_hides_own_shadow_inside_glass() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let mut scene = shadow_scene(1.);
            let pixels = render(&mut renderer, &mut scene);
            assert!(pixel(&pixels, 100, 100)[0] >= 250);
            assert!(pixel(&pixels, 40, 100)[0] < 230);
        });
    }

    #[test]
    fn backdrop_with_zero_opacity_leaves_pixels_untouched() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            let original = render(&mut renderer, &mut stripe_scene());
            let mut scene = stripe_scene();
            let (capture, mut composite) = glass(1.)
                .pair()
                .expect("glass must produce a backdrop pair");
            composite.opacity = 0.;
            scene.insert_primitive(capture);
            scene.insert_primitive(composite);
            assert_eq!(render(&mut renderer, &mut scene), original);
        });
    }

    #[test]
    fn frame_without_backdrop_keeps_no_scratch() {
        objc::rc::autoreleasepool(|| {
            let mut renderer = renderer();
            assert!(renderer.layer.framebuffer_only());
            let mut scene = stripe_scene();
            insert_glass(&mut scene, glass(1.));
            render(&mut renderer, &mut scene);
            assert!(!renderer.backdrop_pool.is_empty());
            assert!(!renderer.blur_kernels.is_empty());
            render(&mut renderer, &mut stripe_scene());
            assert!(renderer.backdrop_pool.is_empty());
            assert!(renderer.blur_kernels.is_empty());
        });
    }
}
