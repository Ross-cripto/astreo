# Astreo

App personal de escritorio en Rust y GPUI.

## Desarrollo en macOS

Instala Rust estable (1.88 o posterior), Xcode y sus herramientas de línea de comandos.
Comprueba que `xcrun --find metal` encuentra el compilador de Metal.

Desde la raíz del proyecto, ejecuta `cargo run --locked` para abrir Astreo.
Usa `cargo test --locked` para ejecutar el objetivo de pruebas de Cargo y
`cargo fmt --check` para revisar el formato.
