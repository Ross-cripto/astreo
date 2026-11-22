use gpui::{
    App, Application, Bounds, Context, TitlebarOptions, Window, WindowAppearance, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};

const APP_NAME: &str = "Astreo";
const INITIAL_WINDOW_WIDTH: f32 = 720.0;
const INITIAL_WINDOW_HEIGHT: f32 = 480.0;

struct AstreoView;

impl Render for AstreoView {
    fn render(&mut self, window: &mut Window, _context: &mut Context<Self>) -> impl IntoElement {
        let (background, foreground) = match window.appearance() {
            WindowAppearance::Light | WindowAppearance::VibrantLight => (0xf8f8f8, 0x171717),
            WindowAppearance::Dark | WindowAppearance::VibrantDark => (0x171717, 0xf8f8f8),
        };

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(background))
            .text_color(rgb(foreground))
            .text_2xl()
            .child(APP_NAME)
    }
}

fn main() {
    Application::new().run(|app: &mut App| {
        let window_size = size(px(INITIAL_WINDOW_WIDTH), px(INITIAL_WINDOW_HEIGHT));
        let bounds = Bounds::centered(None, window_size, app);

        app.on_window_closed(|app| {
            if app.windows().is_empty() {
                app.quit();
            }
        })
        .detach();

        app.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(APP_NAME.into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, app| app.new(|_| AstreoView),
        )
        .expect("Failed to open the Astreo window");

        app.activate(false);
    });
}
