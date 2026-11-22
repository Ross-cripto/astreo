# Astreo

App de escritorio hecha con GPUI.

- Proyecto personal en `~/Code/personales/astreo`.
- Stack de Rust (edición 2024), Cargo y GPUI 0.2.2.
- Arranque con `cargo run --locked`.
- Pruebas con `cargo test --locked`.

## Diseño

Toda la interfaz sigue `DESIGN.md`, que es normativo: colores, tipografía, medidas y componentes.

- Cada pantalla aprobada tiene un render en `docs/design/`. Míralo antes de construirla.
- Usa los recursos de `assets/` tal cual, sin redibujarlos:
  - logo e icono de la app en `assets/brand/`;
  - iconos de 16 px en `assets/icons/`;
  - marcas de Claude Code y Codex en `assets/agents/`.
