use std::borrow::Cow;

use astreo::ui::glass::{Glass, glass, reduce_transparency};
use gpui::{
    App, Application, AssetSource, Bounds, Context, IntoElement, Render, Result, SharedString,
    TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, deferred,
    div, point, prelude::*, px, rgb, rgba, size, svg,
};

const CODE_LINE_COUNT: usize = 150;
const BOTTOM_INSET: f32 = 18.0;
const INITIAL_CODE_OFFSET: usize = 2;
const CODE_CYCLE_LENGTH: usize = 12;
const ASSETS: &[(&str, &[u8])] = &[
    (
        "icons/search.svg",
        include_bytes!("../assets/icons/search.svg"),
    ),
    (
        "icons/check.svg",
        include_bytes!("../assets/icons/check.svg"),
    ),
    (
        "icons/branch.svg",
        include_bytes!("../assets/icons/branch.svg"),
    ),
    (
        "agents/claude.svg",
        include_bytes!("../assets/agents/claude.svg"),
    ),
];

struct DemoAssets;

impl AssetSource for DemoAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(ASSETS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(ASSETS
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

struct Cristal {
    show_live: bool,
}

const CODE: u32 = 0xd6d6d6;
const KEYWORD: u32 = 0xc4a1ff;
const STRING: u32 = 0xf2b482;
const FUNCTION: u32 = 0x8ab4ff;
const TYPE: u32 = 0x7fd8c4;
const NUMBER: u32 = 0xf7a07a;
const PUNCTUATION: u32 = 0xa9a9a9;

fn code_cycle_index(index: usize) -> usize {
    (index + INITIAL_CODE_OFFSET) % CODE_CYCLE_LENGTH
}

fn code_tokens(index: usize) -> &'static [(&'static str, u32)] {
    match code_cycle_index(index) {
        0 => &[
            ("export async function ", KEYWORD),
            ("refreshSession", FUNCTION),
            ("(", PUNCTUATION),
            ("request", CODE),
            (": ", PUNCTUATION),
            ("Request", TYPE),
            (") {", PUNCTUATION),
        ],
        1 => &[
            ("  const ", KEYWORD),
            ("token ", CODE),
            ("= ", PUNCTUATION),
            ("parseBearer", FUNCTION),
            ("(", PUNCTUATION),
            ("request.headers.authorization", CODE),
            (");", PUNCTUATION),
        ],
        2 => &[
            ("  if ", KEYWORD),
            ("(!", PUNCTUATION),
            ("token", CODE),
            (") ", PUNCTUATION),
            ("return ", KEYWORD),
            ("response.", CODE),
            ("status", FUNCTION),
            ("(", PUNCTUATION),
            ("401", NUMBER),
            (");", PUNCTUATION),
        ],
        3 => &[
            ("  const ", KEYWORD),
            ("session ", CODE),
            ("= ", PUNCTUATION),
            ("await ", KEYWORD),
            ("store.", CODE),
            ("find", FUNCTION),
            ("(", PUNCTUATION),
            ("token", CODE),
            (");", PUNCTUATION),
        ],
        4 => &[
            ("  await ", KEYWORD),
            ("rateLimit.", CODE),
            ("check", FUNCTION),
            ("(", PUNCTUATION),
            ("session.userId", CODE),
            (");", PUNCTUATION),
        ],
        5 => &[
            ("  return ", KEYWORD),
            ("response.", CODE),
            ("json", FUNCTION),
            ("({ ", PUNCTUATION),
            ("session, refreshed", CODE),
            (": ", PUNCTUATION),
            ("true", KEYWORD),
            (" });", PUNCTUATION),
        ],
        6 | 11 => &[("}", PUNCTUATION)],
        7 => &[],
        8 => &[
            ("function ", KEYWORD),
            ("parseBearer", FUNCTION),
            ("(", PUNCTUATION),
            ("header", CODE),
            (": ", PUNCTUATION),
            ("string", TYPE),
            ("): ", PUNCTUATION),
            ("string", TYPE),
            (" | ", PUNCTUATION),
            ("null ", KEYWORD),
            ("{", PUNCTUATION),
        ],
        9 => &[
            ("  if ", KEYWORD),
            ("(!", PUNCTUATION),
            ("header?.", CODE),
            ("startsWith", FUNCTION),
            ("(", PUNCTUATION),
            ("\"Bearer \"", STRING),
            (")) ", PUNCTUATION),
            ("return null", KEYWORD),
            (";", PUNCTUATION),
        ],
        _ => &[
            ("  return ", KEYWORD),
            ("header.", CODE),
            ("slice", FUNCTION),
            ("(", PUNCTUATION),
            ("7", NUMBER),
            (").", PUNCTUATION),
            ("trim", FUNCTION),
            ("();", PUNCTUATION),
        ],
    }
}

fn code_line(index: usize) -> impl IntoElement {
    div()
        .relative()
        .flex()
        .h(px(22.))
        .items_center()
        .whitespace_nowrap()
        .when((3..=5).contains(&code_cycle_index(index)), |row| {
            row.bg(rgba(0x10b98112)).child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(2.))
                    .bg(rgb(0x10b981)),
            )
        })
        .child(
            div()
                .w(px(64.))
                .flex_shrink_0()
                .pr(px(16.))
                .text_right()
                .text_color(rgb(0x575757))
                .child((index + 1).to_string()),
        )
        .children(
            code_tokens(index)
                .iter()
                .map(|(text, color)| div().text_color(rgb(*color)).child(*text)),
        )
}

fn menu_summary(
    label: Option<&'static str>,
    changes: Option<(&'static str, &'static str)>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .flex_shrink_0()
        .text_size(px(12.))
        .text_color(rgb(0x7a7a7a))
        .when_some(label, |summary, label| summary.child(label))
        .when_some(changes, |summary, (added, removed)| {
            summary.child(
                div()
                    .flex()
                    .gap(px(6.))
                    .font_family("Menlo")
                    .text_size(px(11.5))
                    .child(div().text_color(rgb(0x6ee7b7)).child(added))
                    .child(div().text_color(rgb(0xfda4af)).child(removed)),
            )
        })
}

fn menu_option(
    icon: &'static str,
    title: &'static str,
    subtitle: Option<&'static str>,
    summary: impl IntoElement,
    highlighted: bool,
    selected: bool,
) -> impl IntoElement {
    let icon_color = if selected { 0xa594ff } else { 0x7a7a7a };
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(10.))
        .h(px(if subtitle.is_some() { 50. } else { 32. }))
        .rounded(px(8.))
        .text_size(px(13.))
        .text_color(rgb(0xebebeb))
        .when(highlighted, |row| row.bg(rgba(0xebebeb17)))
        .child(
            svg()
                .path(icon)
                .size(px(16.))
                .flex_shrink_0()
                .text_color(rgb(icon_color)),
        )
        .child(div().flex_1().flex().flex_col().child(title).when_some(
            subtitle,
            |column, subtitle| {
                column.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0x8a8a8a))
                        .child(subtitle),
                )
            },
        ))
        .child(summary)
}

fn menu(reduced: bool) -> impl IntoElement {
    glass(div(), Glass::Menu, reduced)
        .absolute()
        .left(px(240.))
        .top(px(40.))
        .text_size(px(13.5))
        .child(
            div()
                .h(px(46.))
                .px(px(16.))
                .flex()
                .items_center()
                .gap(px(10.))
                .border_b(px(0.5))
                .border_color(rgba(0xffffff14))
                .child(
                    svg()
                        .path("icons/search.svg")
                        .size(px(16.))
                        .text_color(rgb(0x7a7a7a)),
                )
                .child(div().w(px(1.5)).h(px(17.)).bg(rgb(0xa594ff)))
                .child(
                    div()
                        .text_color(rgb(0x6e6e6e))
                        .child("Buscar una rama, un commit o un worktree"),
                ),
        )
        .child(
            div()
                .p(px(6.))
                .flex()
                .flex_col()
                .child(menu_option(
                    "icons/check.svg",
                    "Sin commitear",
                    Some("Lo que aún no está en ningún commit"),
                    menu_summary(Some("8 archivos ·"), Some(("+186", "−24"))),
                    false,
                    true,
                ))
                .child(menu_option(
                    "icons/branch.svg",
                    "Toda la rama frente a main",
                    Some("Sus 3 commits y lo que aún no está en ninguno"),
                    menu_summary(Some("8 archivos"), None),
                    true,
                    false,
                ))
                .child(
                    div()
                        .h(px(26.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .text_size(px(11.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(0x6e6e6e))
                        .child("Otro worktree"),
                )
                .child(menu_option(
                    "icons/branch.svg",
                    "fix/session-expiry",
                    None,
                    menu_summary(None, Some(("+42", "−9"))),
                    false,
                    false,
                ))
                .child(menu_option(
                    "icons/branch.svg",
                    "main",
                    None,
                    menu_summary(Some("sin cambios"), None),
                    false,
                    false,
                )),
        )
        .child(
            div()
                .h(px(38.))
                .px(px(14.))
                .flex()
                .items_center()
                .border_t(px(0.5))
                .border_color(rgba(0xffffff14))
                .text_size(px(11.5))
                .text_color(rgb(0x969696))
                .child("↑↓ Elegir   ·   ↵ Comparar   ·   ⌘0 Volver a sin commitear"),
        )
}

impl Cristal {
    fn pill(&self, reduced: bool, context: &mut Context<Self>) -> gpui::Stateful<gpui::Div> {
        let kind = if self.show_live {
            Glass::LivePill
        } else {
            Glass::ShortcutPill
        };
        let pill = glass(div().id("glass-pill"), kind, reduced)
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(12.))
            .text_color(rgb(0xcfcfcf))
            .whitespace_nowrap()
            .cursor_pointer()
            // Demo-only hover makes the pill swap discoverable.
            .hover(|style| style.border_color(rgba(0xffffff38)))
            // Demo-only swap lets both materials be compared without simulating live updates.
            .on_click(context.listener(|view, _, _, context| {
                view.show_live = !view.show_live;
                context.notify();
            }));
        if self.show_live {
            return pill
                .child(
                    svg()
                        .path("agents/claude.svg")
                        .size(px(14.))
                        .text_color(rgb(0xd97757)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .child("Claude Code cambió")
                        .child(live_filename("server.ts,"))
                        .child("que ya habías visto, y")
                        .child(live_filename("rateLimit.ts")),
                )
                .child(
                    div()
                        .h(px(26.))
                        .px(px(10.))
                        .rounded(px(7.))
                        .bg(rgba(0xebebeb14))
                        .text_color(rgb(0xebebeb))
                        .flex()
                        .items_center()
                        .child("Ver lo nuevo ⌘J"),
                );
        }
        pill.child("⌥ Mantén para ver la versión anterior · Espacio Siguiente sección")
    }
}

fn live_filename(filename: &'static str) -> impl IntoElement {
    div()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(0xebebeb))
        .child(filename)
}

impl Render for Cristal {
    fn render(&mut self, _window: &mut Window, context: &mut Context<Self>) -> impl IntoElement {
        let reduced = reduce_transparency();
        let pill = self.pill(reduced, context);

        div()
            .relative()
            .size_full()
            .bg(rgba(0x0d0d0dd1))
            .text_color(rgb(0xebebeb))
            .font_family(".SystemUIFont")
            .text_size(px(12.))
            .child(
                div()
                    .h(px(54.))
                    .pl(px(92.))
                    .pr(px(16.))
                    .justify_between()
                    .flex()
                    .items_center()
                    .text_color(rgb(0x969696))
                    .child("Astreo · Cristal sobre código"),
            )
            .child(
                div()
                    .absolute()
                    .top(px(54.))
                    .bottom(px(8.))
                    .left(px(8.))
                    .right(px(8.))
                    .rounded(px(10.))
                    .overflow_hidden()
                    .bg(rgba(0x141414db))
                    .child(
                        div()
                            .id("code-pane")
                            .size_full()
                            .overflow_y_scroll()
                            // Menlo stands in for JetBrains Mono in this demo.
                            .font_family("Menlo")
                            .text_size(px(13.))
                            .py(px(12.))
                            .children((0..CODE_LINE_COUNT).map(code_line)),
                    ),
            )
            .child(deferred(menu(reduced)))
            .child(deferred(
                div()
                    .absolute()
                    .left(px(8.))
                    .right(px(8.))
                    .bottom(px(BOTTOM_INSET + 8.))
                    .flex()
                    .justify_center()
                    .child(pill),
            ))
    }
}

fn main() {
    Application::new()
        .with_assets(DemoAssets)
        .run(|app: &mut App| {
            app.on_window_closed(|app| {
                if app.windows().is_empty() {
                    app.quit();
                }
            })
            .detach();
            app.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1100.), px(720.)),
                        app,
                    ))),
                    window_background: WindowBackgroundAppearance::Blurred,
                    titlebar: Some(TitlebarOptions {
                        title: Some("Astreo · Cristal".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(16.), px(16.))),
                    }),
                    ..Default::default()
                },
                |window, app| {
                    app.new(|context| {
                        context
                            .observe_window_activation(window, |_, _, context| context.notify())
                            .detach();
                        Cristal { show_live: false }
                    })
                },
            )
            .expect("Failed to open the glass demo");
            app.activate(false);
        });
}
