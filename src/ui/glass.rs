use gpui::{BackdropBlur, BoxShadow, Hsla, Rgba, Styled, point, px, rgba};

const MENU_FILL: u32 = 0x222224db;
const MENU_OPAQUE_FILL: u32 = 0x222224f7;
const PILL_FILL: u32 = 0x28282ae6;
const PILL_OPAQUE_FILL: u32 = 0x28282af5;
const MENU_BLUR: f32 = 30.0;
const MENU_SATURATION: f32 = 1.5;
const PILL_BLUR: f32 = 24.0;
const PILL_SATURATION: f32 = 1.4;
const BORDER_WIDTH: f32 = 0.5;
const MENU_WIDTH: f32 = 468.;
const MENU_RADIUS: f32 = 12.;
const MENU_BORDER_ALPHA: f32 = 0.14;
const PILL_BORDER_ALPHA: f32 = 0.12;
const MENU_SHADOWS: [BoxShadow; 2] = [shadow(24., 60., 0.55), shadow(6., 16., 0.30)];
const PILL_SHADOW: BoxShadow = shadow(12., 32., 0.50);

struct PillSpec {
    height: f32,
    radius: f32,
    padding_left: f32,
    padding_right: f32,
}

const SHORTCUT_PILL: PillSpec = PillSpec {
    height: 28.,
    radius: 9.,
    padding_left: 6.,
    padding_right: 10.,
};
const LIVE_PILL: PillSpec = PillSpec {
    height: 36.,
    radius: 11.,
    padding_left: 12.,
    padding_right: 5.,
};

/// The three surfaces approved for in-window glass in DESIGN.md.
#[derive(Clone, Copy, Debug)]
pub enum Glass {
    Menu,
    ShortcutPill,
    LivePill,
}

/// Material values independent of element geometry, including the accessibility fallback.
#[derive(Debug, PartialEq)]
pub struct GlassStyle {
    pub fill: Rgba,
    pub blur: Option<BackdropBlur>,
}

/// Resolves the approved material without reading global accessibility state.
pub fn glass_style(kind: Glass, reduce_transparency: bool) -> GlassStyle {
    let (fill, opaque_fill, radius, saturation) = match kind {
        Glass::Menu => (MENU_FILL, MENU_OPAQUE_FILL, MENU_BLUR, MENU_SATURATION),
        Glass::ShortcutPill | Glass::LivePill => {
            (PILL_FILL, PILL_OPAQUE_FILL, PILL_BLUR, PILL_SATURATION)
        }
    };
    if reduce_transparency {
        return GlassStyle {
            fill: rgba(opaque_fill),
            blur: None,
        };
    }
    GlassStyle {
        fill: rgba(fill),
        blur: Some(BackdropBlur {
            radius: px(radius),
            saturation,
        }),
    }
}

const fn shadow(offset_y: f32, blur_radius: f32, opacity: f32) -> BoxShadow {
    BoxShadow {
        color: Hsla {
            h: 0.,
            s: 0.,
            l: 0.,
            a: opacity,
        },
        offset: point(px(0.), px(offset_y)),
        blur_radius: px(blur_radius),
        spread_radius: px(0.),
    }
}

fn pill<E: Styled>(element: E, spec: &PillSpec) -> E {
    element
        .h(px(spec.height))
        .rounded(px(spec.radius))
        .pl(px(spec.padding_left))
        .pr(px(spec.padding_right))
        .border_color(gpui::white().opacity(PILL_BORDER_ALPHA))
        .shadow(vec![PILL_SHADOW])
}

/// Applies the material, dimensions, border and shadow of an approved glass surface.
pub fn glass<E: Styled>(element: E, kind: Glass, reduce_transparency: bool) -> E {
    let material = glass_style(kind, reduce_transparency);
    let mut element = element.bg(material.fill).border(px(BORDER_WIDTH));
    element = match kind {
        Glass::Menu => element
            .w(px(MENU_WIDTH))
            .rounded(px(MENU_RADIUS))
            .border_color(gpui::white().opacity(MENU_BORDER_ALPHA))
            .shadow(MENU_SHADOWS.to_vec()),
        Glass::ShortcutPill => pill(element, &SHORTCUT_PILL),
        Glass::LivePill => pill(element, &LIVE_PILL),
    };
    if let Some(blur) = material.blur {
        return element.backdrop_blur(blur.radius, blur.saturation);
    }
    // Clear any blur applied earlier so the accessibility fallback cannot retain it.
    element.style().backdrop_blur = None;
    element
}

/// Reads macOS Reduce Transparency; callers re-read it when the window activates.
// objc macros expand a legacy cargo-clippy cfg that this crate does not declare.
#[allow(unexpected_cfgs)]
pub fn reduce_transparency() -> bool {
    use objc::{class, msg_send, sel, sel_impl};
    // NSWorkspace is shared by AppKit; neither message transfers ownership.
    unsafe {
        let workspace: *mut objc::runtime::Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let reduced: objc::runtime::BOOL =
            msg_send![workspace, accessibilityDisplayShouldReduceTransparency];
        reduced != objc::runtime::NO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glass_applies_material_per_kind() {
        for (kind, fill, opaque, radius, saturation) in [
            (Glass::Menu, 0x222224db, 0x222224f7, 30., 1.5),
            (Glass::ShortcutPill, 0x28282ae6, 0x28282af5, 24., 1.4),
            (Glass::LivePill, 0x28282ae6, 0x28282af5, 24., 1.4),
        ] {
            let mut transparent = glass(gpui::div(), kind, false);
            assert_eq!(transparent.style().background, Some(rgba(fill).into()));
            assert_eq!(
                transparent.style().backdrop_blur,
                Some(BackdropBlur {
                    radius: px(radius),
                    saturation,
                })
            );
            let mut reduced = glass(transparent, kind, true);
            assert_eq!(reduced.style().background, Some(rgba(opaque).into()));
            assert_eq!(reduced.style().backdrop_blur, None);
        }
    }

    #[test]
    fn reduce_transparency_drops_blur_and_uses_opaque_fill() {
        for reduced in [false, true] {
            assert_eq!(
                glass_style(Glass::ShortcutPill, reduced),
                glass_style(Glass::LivePill, reduced)
            );
        }
        for (kind, opaque, transparent, radius, saturation) in [
            (Glass::Menu, 0x222224f7, 0x222224db, 30., 1.5),
            (Glass::ShortcutPill, 0x28282af5, 0x28282ae6, 24., 1.4),
        ] {
            let fallback = glass_style(kind, true);
            assert_eq!(fallback.fill, rgba(opaque));
            assert_eq!(fallback.blur, None);
            let glass = glass_style(kind, false);
            assert_eq!(glass.fill, rgba(transparent));
            assert_eq!(
                glass.blur,
                Some(BackdropBlur {
                    radius: px(radius),
                    saturation
                })
            );
        }
    }
}
