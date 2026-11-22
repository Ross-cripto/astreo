---
name: Astreo
description: El código lo escribe tu agente. El criterio es tuyo.
colors:
  lavender: "#a594ff"
  lavender-deep: "#6a5ae6"
  on-lavender: "#ffffff"
  note-fill: "rgba(165,148,255,0.10)"
  note-edge: "rgba(165,148,255,0.24)"
  note-ink: "#ecebff"
  window: "rgba(13,13,13,0.82)"
  pane: "rgba(20,20,20,0.86)"
  popover-glass: "rgba(34,34,36,0.86)"
  float-glass: "rgba(40,40,42,0.90)"
  popover: "rgba(34,34,36,0.97)"
  float: "rgba(40,40,42,0.96)"
  node: "rgba(34,34,34,0.95)"
  node-selected: "rgba(52,52,52,0.98)"
  well: "#0b0b0e"
  fill-subtle: "rgba(235,235,235,0.06)"
  fill: "rgba(235,235,235,0.08)"
  fill-strong: "rgba(235,235,235,0.12)"
  hairline: "rgba(255,255,255,0.08)"
  window-edge: "rgba(255,255,255,0.13)"
  ink-bright: "#f5f5f5"
  ink-dim: "#757981"
  ink: "#ebebeb"
  ink-row: "#cfcfcf"
  muted: "#969696"
  faint: "#6e6e6e"
  line-number: "#575757"
  code: "#d6d6d6"
  code-context: "#7d7d7d"
  added: "#10b981"
  added-ink: "#6ee7b7"
  added-line: "rgba(16,185,129,0.07)"
  added-word: "rgba(16,185,129,0.14)"
  removed-ink: "#fda4af"
  ghost-ink: "rgba(253,164,175,0.74)"
  ghost-strike: "rgba(244,63,94,0.70)"
  modified: "#f5b453"
  modified-fill: "rgba(245,180,83,0.12)"
  syntax-keyword: "#c4a1ff"
  syntax-string: "#f2b482"
  syntax-function: "#8ab4ff"
  syntax-type: "#7fd8c4"
  syntax-number: "#f7a07a"
  syntax-punctuation: "#a9a9a9"
  agent-claude: "#d97757"
  agent-codex: "#e8e9ed"
  logo-ink: "#f4f3f8"
  icon-top: "#201f29"
  icon-bottom: "#0a0a0e"
typography:
  display:
    fontFamily: ".SystemUIFont, SF Pro Display"
    fontSize: "32px"
    fontWeight: 600
    lineHeight: 1.15
    letterSpacing: "-0.024em"
  title-lg:
    fontFamily: ".SystemUIFont, SF Pro Display"
    fontSize: "19px"
    fontWeight: 600
    letterSpacing: "-0.015em"
  title:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "17px"
    fontWeight: 600
    letterSpacing: "-0.012em"
  heading:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "15px"
    fontWeight: 600
    letterSpacing: "-0.01em"
  body-lg:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.55
  body:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "-0.003em"
  label:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "12.5px"
    fontWeight: 500
  caption:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "12px"
    fontWeight: 400
  micro:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "11px"
    fontWeight: 600
  kbd:
    fontFamily: ".SystemUIFont, SF Pro Text"
    fontSize: "10.5px"
    fontWeight: 600
  code:
    fontFamily: "JetBrains Mono"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "22px"
    fontFeature: "calt 0"
  code-focus:
    fontFamily: "JetBrains Mono"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: "25px"
    fontFeature: "calt 0"
  code-ghost:
    fontFamily: "JetBrains Mono"
    fontSize: "11.5px"
    fontWeight: 400
    lineHeight: "14px"
  code-meta:
    fontFamily: "JetBrains Mono"
    fontSize: "12px"
    fontWeight: 400
  code-symbol:
    fontFamily: "JetBrains Mono"
    fontSize: "13px"
    fontWeight: 600
  wordmark:
    fontFamily: "Geist"
    fontWeight: 600
    letterSpacing: "-0.035em"
rounded:
  xs: "4px"
  sm: "5px"
  md: "6px"
  row: "7px"
  control: "8px"
  pill: "9px"
  panel: "10px"
  float: "11px"
  window: "12px"
  card: "14px"
  hero: "16px"
spacing:
  xxs: "2px"
  xs: "4px"
  sm: "6px"
  md: "8px"
  base: "10px"
  lg: "12px"
  xl: "16px"
  2xl: "20px"
  3xl: "24px"
  4xl: "32px"
components:
  window:
    backgroundColor: "{colors.window}"
    rounded: "{rounded.window}"
  titlebar:
    height: "44px"
  pane:
    backgroundColor: "{colors.pane}"
    rounded: "{rounded.panel}"
  pane-header:
    height: "52px"
    padding: "0 16px 0 20px"
  rail:
    width: "256px"
    padding: "10px 10px 14px 12px"
  margin-column:
    width: "336px"
  row-file:
    textColor: "{colors.ink-row}"
    typography: "{typography.body}"
    rounded: "{rounded.row}"
    height: "28px"
    padding: "0 8px"
  row-file-seen:
    textColor: "{colors.muted}"
  row-file-current:
    backgroundColor: "{colors.fill}"
    textColor: "{colors.ink}"
  row-section:
    textColor: "{colors.muted}"
    typography: "{typography.caption}"
    rounded: "{rounded.row}"
    height: "26px"
    padding: "0 8px 0 32px"
  plate:
    textColor: "{colors.ink}"
    rounded: "{rounded.row}"
    height: "28px"
    padding: "0 9px 0 6px"
  plate-open:
    backgroundColor: "{colors.fill}"
  segmented:
    backgroundColor: "{colors.fill-subtle}"
    rounded: "{rounded.control}"
    padding: "2px"
  segmented-item:
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    height: "24px"
    padding: "0 12px"
  segmented-item-on:
    backgroundColor: "{colors.fill-strong}"
    textColor: "{colors.ink}"
  button-primary:
    backgroundColor: "{colors.lavender-deep}"
    textColor: "{colors.on-lavender}"
    rounded: "{rounded.control}"
    height: "32px"
    padding: "0 7px 0 14px"
  button-primary-large:
    backgroundColor: "{colors.lavender-deep}"
    textColor: "{colors.on-lavender}"
    rounded: "{rounded.pill}"
    height: "34px"
    padding: "0 7px 0 16px"
  button-secondary:
    backgroundColor: "{colors.fill}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.row}"
    height: "28px"
    padding: "0 6px 0 11px"
  button-text:
    textColor: "{colors.ink}"
    typography: "{typography.label}"
  button-icon:
    textColor: "{colors.muted}"
    rounded: "{rounded.md}"
    size: "24px"
  kbd:
    backgroundColor: "rgba(235,235,235,0.07)"
    textColor: "{colors.muted}"
    typography: "{typography.kbd}"
    rounded: "{rounded.xs}"
    height: "18px"
    padding: "0 5px"
  badge-new:
    backgroundColor: "{colors.fill}"
    textColor: "{colors.ink-row}"
    typography: "{typography.kbd}"
    rounded: "{rounded.sm}"
    padding: "1px 6px"
  badge-changed:
    backgroundColor: "{colors.modified-fill}"
    textColor: "{colors.modified}"
    typography: "{typography.kbd}"
    rounded: "{rounded.sm}"
    padding: "1px 6px"
  note:
    backgroundColor: "{colors.note-fill}"
    textColor: "{colors.note-ink}"
    rounded: "{rounded.panel}"
    width: "600px"
  menu:
    backgroundColor: "{colors.popover-glass}"
    rounded: "{rounded.window}"
    width: "468px"
  menu-option:
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "32px"
    padding: "5px 10px"
  menu-option-highlight:
    backgroundColor: "rgba(235,235,235,0.09)"
  hint-pill:
    backgroundColor: "{colors.float-glass}"
    textColor: "{colors.ink-row}"
    typography: "{typography.caption}"
    rounded: "{rounded.pill}"
    height: "28px"
    padding: "0 10px 0 6px"
  live-pill:
    backgroundColor: "{colors.float-glass}"
    textColor: "{colors.ink-row}"
    rounded: "{rounded.float}"
    height: "36px"
    padding: "0 5px 0 12px"
  focus-card:
    backgroundColor: "rgba(20,20,20,0.88)"
    rounded: "{rounded.card}"
  map-node:
    backgroundColor: "{colors.node}"
    rounded: "{rounded.panel}"
    width: "236px"
    height: "54px"
  map-node-selected:
    backgroundColor: "{colors.node-selected}"
  worktree-card:
    backgroundColor: "{colors.pane}"
    rounded: "{rounded.window}"
  progress-segment:
    backgroundColor: "{colors.fill-strong}"
    rounded: "2px"
    height: "4px"
---

# Design System: Astreo

## Overview

**Creative North Star: "El crepúsculo del revisor"**

Astreo es la hora en que se mira lo que hicieron otros. El agente escribe y tú decides. La interfaz es una ventana de cristal oscuro sobre el escritorio, el código ocupa casi todo, y lo único con color propio es lo que importa para decidir: qué cambió (verde, ámbar y rosa) y lo que es tuyo (lavanda). Todo lo demás son grises sobre cristal.

El acabado parte de Monocode como inspiración visual (D32): cristal translúcido, tipografía compacta, filas de 28 px y bordes de medio píxel. Ni una línea de su código (D13). De la marca se quedan el lavanda, la tipografía y el botón principal (D37). La app es solo oscura (D16) y nativa de macOS: semáforo, atajos con ⌘ y vibrancy.

El logo es el Eclipse: dos discos iguales, el de antes punteado y por detrás, y lo que asoma del de ahora es lo que cambió (D34). La misma idea aparece en la interfaz: lo de antes, tachado y tenue, encima de lo de ahora.

**Key Characteristics:**
- Cristal oscuro en todas las ventanas; los paneles son una segunda capa de cristal, no fondos opacos.
- Densidad de herramienta: texto base de 13 px, filas de 26 a 32 px y bordes de 0,5 px.
- Color solo con significado: los colores del diff para el cambio y el lavanda para el revisor.
- Teclado primero: cada acción visible enseña su atajo.
- Nada se mueve solo: lo que llega en vivo se anuncia, no desplaza la vista.

**Referencias.** Las pantallas aprobadas están en `docs/design/` (renders a 1600×1000 sobre el fondo de macOS). El lienzo original con todos los artboards es privado y solo lo abre el dueño del proyecto.

| Render | Pantalla | Decisión |
|---|---|---|
| `01-fantasma.png` | Vista fantasma | D32 |
| `02-mapa.png`, `03-mapa-grande.png` | Mapa, cambio pequeño y grande | D33 |
| `04-inicio.png` | Inicio | D36, D38 |
| `05-introduccion.png` | Introducción de primera vez | D44 |
| `06-comparar-menu.png`, `07-comparando.png` | Comparar con | D40 |
| `08-en-vivo.png` | En vivo | D41 |
| `09-estados.png` | Sin cambios y carpeta sin git | D42 |
| `10-una-a-una.png`, `11-resumen.png` | Una decisión a la vez y resumen (F2) | D43 |

## Colors

Grises neutros sobre cristal, tres familias semánticas para el diff y un único acento lavanda que pertenece al revisor.

### Primary
- **Lavanda del revisor** (lavender): todo lo que es tuyo: el progreso de lo revisado, las notas, el cursor del buscador, la selección de texto (al 35 %) y los segmentos "bien" del resumen.
- **Lavanda profundo** (lavender-deep): solo el fondo del botón principal, con texto blanco (on-lavender). Hay como mucho un botón principal por vista.
- **Nota** (note-fill, note-edge, note-ink): la tarjeta de nota y las notas del resumen. Es el lavanda a baja opacidad con texto casi blanco teñido.

### Secondary
- **Añadido** (added, added-ink, added-line, added-word): la barra de 2 px y el fondo de las líneas nuevas, las cifras `+N` y la palabra resaltada dentro de una línea cambiada. El punto pulsante de "en vivo" también es este verde.
- **Modificado** (modified, modified-fill): la barra de 2 px de las líneas modificadas, la letra `M` del lateral y la etiqueta "cambió".
- **Antes** (removed-ink, ghost-ink, ghost-strike): las cifras `−N` y el texto fantasma de lo anterior, tachado con una línea de 1 px.

### Tertiary
- **Agentes** (agent-claude, agent-codex): solo para la marca del agente junto a su nombre. Claude Code va en su naranja y Codex en blanco tiza.
- **Resaltado de sintaxis** (syntax-*): palabra clave violeta, cadena melocotón, función azul, tipo turquesa, número coral y puntuación gris. Todo es tenue para que el diff mande sobre la sintaxis.

### Neutral
- **Cristal de ventana** (window): fondo de toda ventana, sobre el desenfoque nativo de macOS.
- **Cristal de panel** (pane): el panel de código, las tarjetas del inicio y los estados vacíos. Va encima de la ventana con un borde interior de 0,5 px de hairline.
- **Flotantes de cristal** (popover-glass, float-glass): el menú "Comparar con" y las dos píldoras de abajo. Son translúcidos y desenfocan el código que tienen detrás (ver Elevation & Depth, D45).
- **Flotantes opacos** (popover, float): los mismos tres cuando macOS tiene activado "Reducir transparencia" (D46).
- **Pozo** (well): fondo opaco de las miniaturas del inicio y del mapa del héroe.
- **Rellenos** (fill-subtle, fill, fill-strong): estados de fila y de control: reposo de un control segmentado, fila actual o placa abierta, y segmento activo.
- **Líneas** (hairline, window-edge): separadores y bordes de 0,5 px. window-edge es el borde interior de la ventana.
- **Tintas** (ink-bright, ink, ink-row, muted, faint, line-number, code, code-context): de más a menos presencia. Van en nombres de símbolo, texto principal, filas, texto secundario, etiquetas y metadatos, números de línea, código cambiado y código de contexto.

### Named Rules
**La Regla del Lavanda.** El lavanda es del revisor, nunca del agente. Si algo lo hizo el agente (el porqué, su marca, sus cambios), no lleva lavanda.

**La Regla del Diff.** El verde, el ámbar y el rosa solo significan añadido, modificado y antes. No se usan para estados de éxito, avisos ni decoración. La única excepción es el punto verde de "en vivo".

## Typography

**Fuente de interfaz:** SF Pro (`.SystemUIFont` en GPUI)
**Fuente de código:** JetBrains Mono, incluida en la app (licencia OFL) y con las ligaduras apagadas (`calt 0`)
**Logotipo:** Geist 600, solo dentro del logo exportado; la app nunca escribe texto en Geist.

**Character:** La fuente del sistema hace que Astreo parezca de macOS. La monoespaciada es de líneas abiertas y se lee bien en diffs largos. Cada fuente tiene su papel: los nombres de símbolo, rutas, hashes y cifras del diff van en mono, y todo lo demás en SF.

### Hierarchy
- **Display** (600, 32 px, 1.15, −0,024 em): titulares del inicio, de la introducción y del resumen. La primera frase va en `ink-dim` y la segunda en `ink`: "Buenas tardes. Tus agentes dejaron 4 cambios por revisar."
- **Title large** (600, 19 px): título de un estado vacío.
- **Title** (600, 17 px): nombre del worktree en el héroe del inicio.
- **Heading** (600, 15 px): encabezados de grupo en el inicio ("Más worktrees").
- **Body large** (400, 14 px, 1.55): el porqué en "Una a una" y el nombre de las tarjetas.
- **Body** (400, 13 px, 1.5): texto base de la interfaz, las filas y el cuerpo de las notas.
- **Label** (500, 12,5 px): botones, segmentos y placas.
- **Caption** (400, 12 px): metadatos, filas de sección y píldoras.
- **Micro** (600, 11 px): etiquetas de grupo del lateral ("Cambios · 8 archivos") y letras de estado (`M`, `A`).
- **Kbd** (600, 10,5 px): teclas y etiquetas "Nuevo" y "cambió".
- **Code** (13 px / 22 px): el diff. En "Una a una" sube a 14 px / 25 px. Lo fantasma va a 11,5 px / 14 px y los números de línea a 12 px.

### Named Rules
**La Regla de las Dos Voces.** Solo hay dos fuentes en pantalla: SF para hablar y mono para el código. El tamaño mínimo es 10,5 px y solo para teclas y etiquetas.

**La Regla de las Cifras.** Las cifras que se comparan (líneas, archivos, progreso) usan dígitos tabulares.

## Layout

Ventana de referencia de 1440×900, que es el tamaño de los renders. Se propone un mínimo de 1024×640; por debajo, el margen del porqué se pliega antes que el código.

- **Barra de título** de 44 px, dibujada por la app y con el semáforo nativo en (16, 16). De izquierda a derecha:
  - la placa del worktree (icono de rama, rama en negrita y chevron);
  - la placa de comparación ("Sin commitear" y chevron);
  - un hueco flexible;
  - el control segmentado de vistas;
  - el estado del agente (punto vivo, marca y "Claude Code escribe en rateLimit.ts");
  - las cifras `+N −N` en mono;
  - el botón de notas.
- **Vista fantasma:**
  - Lateral de 256 px, sin fondo propio: es la ventana.
  - Panel de código desde x = 256, con 8 px de aire a la derecha y abajo, radio de 10 y cabecera de 52 px.
  - El cuerpo es una rejilla de dos columnas: el código, flexible, y el margen del porqué, de 336 px con un borde izquierdo de hairline.
- **Rejilla del código:** cuatro columnas fijas.
  - 46 px para números de línea, alineados a la derecha con 10 px de aire.
  - 2 px para la barra de cambio.
  - 10 px de aire.
  - El código.

  Las cabeceras de sección miden 34 px y las líneas 22 px. Notas y pliegues empiezan en x = 58 para alinearse con el código.
- **Mapa:**
  - Las historias van en carriles de 240 px con radio 12 y las capas en filas de 104 px, alternando el relleno de fila con blanco al 1,8 %.
  - Los nodos miden 236×54.
  - En cambios grandes el mapa se desplaza en horizontal: una regla de historias arriba y un desvanecido de 110 px a la derecha (D33).
- **Una a una:** una columna centrada con 250 px de margen a cada lado. La sección anterior y la siguiente asoman plegadas como franjas de 42 px.
- **Resumen:** columna centrada con 310 px de margen a cada lado.
- **Inicio:** columna con 160 px de margen a cada lado. El héroe mide 290 px de alto y debajo hay una rejilla de 4 tarjetas con 14 px de separación.
- **Estados vacíos:** contenido centrado en el panel. Glifo de 84 px, título a 22 px del glifo, texto de 440 px de ancho y acciones a 22 px.
- **Ritmo:** los espaciados vienen de la escala (2, 4, 6, 8, 10, 12, 16, 20, 24 y 32). Dentro de un control el hueco es de 6 a 9 px y entre bloques, de 16 a 34 px.

## Elevation & Depth

La profundidad viene de capas de cristal, no de sombras. Ventana, panel y flotante son tres capas de cristal del mismo gris, y cada una desenfoca lo que tiene detrás. Las sombras solo aparecen en lo que flota sobre el código: menú, píldoras, tarjeta de foco y nodos del mapa.

**En GPUI:**
- La ventana usa `WindowBackgroundAppearance::Blurred` y la barra de título transparente. El fondo `window` va en el `div` raíz.
- **Cristal dentro de la ventana** (D45): Astreo lleva una copia de GPUI 0.2.2 en `vendor/gpui` con un cambio en el motor. `.backdrop_blur(radio, saturación)` equivale a `backdrop-filter: blur() saturate()` de CSS, y solo se usa a través de `src/ui/glass.rs`:

  | Superficie | Fondo | Desenfoque | Saturación |
  |---|---|---|---|
  | Menú "Comparar con" | `popover-glass` | 30 px | 1,5 |
  | Píldora de atajos (D47) | `float-glass` | 24 px | 1,4 |
  | Píldora en vivo | `float-glass` | 24 px | 1,4 |

  La sombra propia del elemento no se ve a través de su cristal, igual que en CSS. Si el elemento se desvanece, su cristal también.
- **Reducir transparencia** (D46): si está activado en macOS, las tres superficies pierden el desenfoque y usan `popover` y `float`. El ajuste se vuelve a leer cuando la ventana recupera el foco.
- Ninguna otra superficie lleva cristal propio.
- `BoxShadow` no admite sombras interiores, así que los bordes interiores de 0,5 px se dibujan como borde normal.
- El brillo superior del botón principal es opcional.

### Shadow Vocabulary
- **Ventana** (`0 40px 90px rgba(0,0,0,.45), 0 12px 28px rgba(0,0,0,.28)`): la pone macOS. La app solo dibuja el borde interior `window-edge` de 0,5 px.
- **Menú** (`0 24px 60px rgba(0,0,0,.55), 0 6px 16px rgba(0,0,0,.30)`): menú "Comparar con". Lleva borde interior de 0,5 px a `rgba(255,255,255,.14)`.
- **Píldora** (`0 12px 32px rgba(0,0,0,.50)`): píldoras de atajos y de en vivo. Lleva borde de 0,5 px a `rgba(255,255,255,.12)`.
- **Tarjeta de foco** (`0 24px 60px rgba(0,0,0,.35)`): la sección grande de "Una a una".
- **Nodo** (`0 6px 18px rgba(0,0,0,.30)`): nodos del mapa. Seleccionado: `0 10px 26px rgba(0,0,0,.45)` con borde de 1 px a `rgba(255,255,255,.30)`.
- **Botón principal** (`0 1px 2px rgba(0,0,0,.40)`): su única sombra.

### Named Rules
**La Regla del Cristal.** Ninguna ventana es opaca (D37). Si un fondo tapa del todo el escritorio, está mal, salvo las miniaturas del inicio (`well`).

## Shapes

Radios pequeños y consistentes que crecen con el tamaño del objeto:
- 4 px en teclas;
- 5 a 7 px en etiquetas, filas y placas;
- 8 a 11 px en botones, paneles y píldoras;
- 12 a 16 px en ventanas, tarjetas y el héroe.

Los bordes son de 0,5 px, salvo la selección, que usa 1 px. Los puntos de estado son círculos de 6 a 7 px.

La forma propia de Astreo es la **muesca de la nota**: un cuadrado de 9 px girado 45° que asoma por encima de la tarjeta, a 18 px del borde izquierdo, y apunta a la línea anotada. El otro motivo es el **disco punteado** del Eclipse, que se repite en el estado "sin cambios".

## Components

### Barra de título y placas
- **Placa:** 28 px de alto, radio 7 y, a la izquierda, un icono en una baldosa de 18 px con radio 5. En reposo no tiene fondo; abierta, `fill`.
- La placa de comparación muestra "Sin commitear". Al comparar dice "Toda la rama frente a **main**" con una × de 14 px que vuelve a sin commitear (D40).
- **Control segmentado:**
  - Fondo `fill-subtle` con borde de 0,5 px a blanco al 6 %.
  - Segmentos de 24 px con icono de 13 px y texto label.
  - El activo lleva `fill-strong`, tinta y una sombra de 1 px.
  - Vistas: Fantasma, Mapa y Una a una.

### Lateral (recorrido)
- Etiqueta de grupo en micro y `faint`, con aire de 10 px arriba y 6 px abajo.
- **Fila de archivo:** 28 px, icono de 15 px, nombre con puntos suspensivos y una columna de estado de 12 px (`M` ámbar, `A` verde o check si está visto).
  - Vista: `muted`.
  - Actual: `fill` y tinta.
  - Si cambió después de verla: pierde el check y muestra la etiqueta "cambió" (D41).
- **Fila de sección:** 26 px, sangrada 32 px, con un punto de 6 px del color de su cambio.
- **Árbol:** guías verticales de 0,5 px a blanco al 10 %.

### Diff fantasma
- **Línea añadida:** fondo `added-line` y barra de 2 px `added`. Línea modificada: solo barra `modified`. Contexto: `code-context`.
- **Lo de antes** va encima de la línea que cambió, en `code-ghost` y `ghost-ink`, tachado con `ghost-strike`. Tiene 4 px de aire arriba y −2 px abajo para pegarse a su línea.
  - Solo aparece donde algo cambió; `⌥` muestra la versión anterior completa.
- **Palabra cambiada** dentro de una línea: fondo `added-word` con radio 2.
- **Cabecera de sección:** 34 px. Símbolo en `code-symbol` y `ink-bright`, y detrás, en caption `faint`, "nueva · líneas 16–28".
- **Pliegue:** "14 líneas sin cambios" en caption `faint`, seguido de una línea de hairline hasta el borde.

### Margen del porqué
- Anclas: un círculo de 8 px con borde de 1 px a tinta al 38 %, sobre el borde del margen.
- Cabecera de 34 px: "Por qué" en negrita 12 px, y detrás la marca del agente, su nombre y "hace 4 min" en `faint`.
- Un conector de 0,5 px enlaza la cabecera con su sección.
- Vistazo de código: fondo negro al 28 %, borde de hairline, radio 8 y texto mono de 11,5 / 18 px en `#8c8c8c`.

### Nota
- **Tarjeta:**
  - fondo `note-fill`, borde interior `note-edge`, radio 10 y ancho máximo de 600 px;
  - margen de 8 px arriba, 24 px a la derecha, 14 px abajo y 58 px a la izquierda (sale de la columna del código);
  - muesca de 9 px a 18 px del borde izquierdo.
- **Cabecera:** avatar de 20 px con degradado `#b9adff`→`#8f7cf6` y silueta blanca, autor, línea y botones de icono de 24 px a la derecha (aprobar, editar, más).
- **Cuerpo:** body en `note-ink` con interlineado de 1,55 y sangría de 40 px.
- **Pie:** 11,5 px a lavanda tenue (`rgba(222,216,255,.5)`) con el destino: "Para el agente · .astreo/review.md".

### Botones y teclas
- **Principal:**
  - lavanda profundo, texto blanco 600 a 13 px y radio 8; a 34 px y radio 9 en "Una a una";
  - su tecla va dentro con fondo blanco al 16 %;
  - uno por vista.
- **Secundario:** `fill`, tinta y label, con 28 px de alto, o 34 px junto a un principal.
- **De texto:** sin fondo, tinta y label, con su tecla al lado. Ejemplos: "Saltar →" y "Guardar .md".
- **Tecla:** caja de 18 px de alto y 18 px de ancho mínimo con radio 4. Lleva sombra inferior de 1 px a negro al 35 % y borde de 0,5 px. Las teclas seguidas se solapan 3 px.
- **Estados** (propuesta por defecto, los mockups solo muestran el reposo):
  - Al pasar el cursor, un paso más de relleno: nada → `fill-subtle` → `fill` → `fill-strong`.
  - Al pulsar, un paso más.
  - El foco de teclado es un anillo de 2 px a lavanda al 60 % con 2 px de separación.

### Etiquetas
- **Nuevo:** worktree descubierto que nunca has abierto (D38). Tipografía kbd, `fill` y `ink-row`.
- **cambió:** archivo visto que el agente volvió a tocar (D41). Tipografía kbd, `modified-fill` y `modified`.
- **Punto vivo:** 6 o 7 px en `added`, con un pulso de 1,6 s (ver Movimiento).

### Menú "Comparar con" (D40)
- **Contenedor:** 468 px de ancho, sale de la placa de comparación. Cristal `popover-glass` (desenfoque 30 px, saturación 1,5), radio 12 y sombra Menú.
- **Buscador:** 46 px, con lupa, cursor lavanda de 1,5 px y texto 13,5 px en `faint`.
- **Grupos:** cabecera de 26 px en micro y `faint`. Opciones de 32 px, o de 50 px con una segunda línea en caption `#8a8a8a`.
  - Cada opción es una rejilla de icono de 16 px, texto y, a la derecha, archivos y `+N −N`.
  - Resaltada: blanco al 9 %. Elegida: check.
- **Orden:**
  1. Sin commitear y Toda la rama frente a main.
  2. Otros worktrees, con la marca y el estado de su agente.
  3. Commits de esta rama, con el hash en mono de 11,5 px.
- **Pie:** 38 px con las teclas "↑↓ Elegir · ↵ Comparar · ⌘0 Volver a sin commitear".

### Píldoras de abajo
- **Ranura:** centrada a 16–18 px del borde inferior del panel. Solo hay una píldora a la vez.
- **Atajos:** 28 px. Por ejemplo, "⌥ Mantén para ver la versión anterior · Espacio Siguiente sección".
- **En vivo:** sustituye a la de atajos (D41). Mide 36 px y su botón secundario 26 px.
  - Texto: "Claude Code cambió **server.ts**, que ya habías visto, y **rateLimit.ts**", con la marca del agente al principio y "Ver lo nuevo ⌘J" al final.
  - Desaparece al pulsarla o al llegar a lo nuevo. Nunca mueve el scroll ni cambia el archivo abierto.

### Una decisión a la vez (D43)
- **Progreso:** "Sección 5 de 12" y una barra de segmentos de 4 px separados 3 px. El segmento vale:
  - `lavender` si está bien;
  - con contorno lavanda de 1 px si tiene nota;
  - blanco al 32 % si se saltó;
  - `ink` si es la actual;
  - `fill-strong` si está pendiente.
- **Tarjeta de foco:**
  - Cabecera de 58 px con el símbolo en mono 14 px.
  - Debajo, el porqué en body-large con comillas `faint`, y luego el código en `code-focus`.
  - Barra de acciones de 72 px: "Saltar →", "Dejar una nota N" y el principal "Bien ↵".
- **Resumen:**
  - Display "Revisado. 12 secciones en 9 minutos.", la misma barra de segmentos con leyenda, las notas para el agente como tarjetas de nota y lo saltado como filas de 46 px con borde de hairline.
  - Acciones: "Dejar las notas al agente ↵" (principal), "Copiar como prompt ⌘C" y "Guardar .md".

### Inicio e introducción
- **Héroe:** cristal de panel con radio 16 y 290 px de alto. A la izquierda, el worktree, el estado vivo, el porqué y el progreso. A la derecha, la constelación del cambio en un pozo de 600 px.
- **Tarjetas de worktree:** cristal de panel con radio 12 y miniatura de 116 px en un pozo con radio 8. Debajo:
  - el nombre con el icono de rama y la hora;
  - el estado con punto vivo, la etiqueta Nuevo y la marca del agente;
  - las partes y `+N −N` en mono.
- **Introducción** (D44): 4 pasos con el mismo cristal.
  - El indicador son segmentos de 22×3 px separados 4 px; el actual en lavanda.
  - Paso 1: Eclipse animado y la frase de marca.
  - Paso 2: cómo se lee un cambio.
  - Paso 3: permiso para leer las conversaciones de los agentes, con sus marcas reales.
  - Paso 4: elegir por dónde empezar, con los worktrees descubiertos y el comando `astreo` opcional.

### Estados vacíos (D42)
- **Sin cambios:** el Eclipse con los dos discos coincidiendo (`assets/brand/astreo-glyph-same.svg`, 84 px), "Nada sin commitear en main", un botón secundario "Comparar con…" y una lista "Donde sí hay cambios" con filas de 36 px.
- **Sin git:** icono de carpeta, "notas-viaje no es un repositorio git", la explicación, el principal "Abrir otra carpeta… ⌘O" y los recientes.

### Mapa (D33)
- **Nodo:** símbolo en mono 12,5 px 600 y una segunda línea en 11,5 px `faint` sangrada 15 px.
- **Variantes:**
  - seleccionado: más claro, con borde de 1 px;
  - fantasma (fuera del cambio): borde discontinuo de 1 px a blanco al 22 % y sin fondo.
- **Relaciones:** líneas de 0,5 a 1 px con etiquetas en caption sobre una pastilla de `rgba(22,22,22,.95)`.
- **Regla de historias:** segmentos de 4 px; el progreso de cada historia en lavanda y la ventana visible en blanco al 7,5 %.
- Las repeticiones se muestran como "repetida en", nunca como porcentaje.

### Iconos
- **Rejilla:** 16 px, trazo de 1,25 a 1,6 px con extremos redondeados, y `currentColor`.
- **Uso en GPUI:** con `svg().path("icons/<nombre>.svg")` y el color con `text_color`. Tamaños de uso: 13, 14, 15 o 16 px.
- **Archivos en `assets/icons/`:**
  - Flechas y navegación: `arrow-down`, `arrow-right`, `arrow-up`, `chevron-down`, `chevron-down-small`, `close`.
  - Objetos: `branch`, `commit`, `file`, `folder`, `lock`, `symbol`, `terminal`, `person`.
  - Revisión y notas: `check`, `more`, `note`, `notes`, `repeated`, `search`, `spark`, `why`.
  - Vistas: `view-ghost`, `view-map`, `view-one`, `view-reading-order`, `view-tree`.
- **Marcas de agente** (`assets/agents/`): `claude.svg` (teñir con `agent-claude`) y `openai.svg` para Codex (teñir con `agent-codex`). Son el trazado de Simple Icons (CC0). Siempre la marca real, nunca un icono genérico de terminal (D39).

### Logo (Eclipse, D34)
- **Construcción:** radio r = 214 sobre un lienzo de 1024. El disco de ahora va en (556, 556) y el de antes en (468, 468), desplazado 0,58 r en diagonal. El de antes lleva 30 trazos (`pathLength 120`, `2.3 1.7`) y queda oculto donde lo tapa el de ahora.
- **Tamaños:** a 32 px, 12 trazos más gruesos; a 16 px, solo la media luna. Cada versión es un archivo distinto; no se escala uno grande hacia abajo.
- **Archivos en `assets/brand/`:**
  - `Astreo.icns`, el icono de la app, de 16 a 1024 px con la variante correcta en cada tamaño;
  - `astreo-icon-1024.png`;
  - `astreo-icon.svg`, `astreo-icon-32.svg` y `astreo-icon-16.svg`;
  - `icon-layers/`, con el fondo y el glifo por separado para Icon Composer, si se quiere el icono con Liquid Glass de macOS 26;
  - `astreo-glyph.svg` (glifo sobre oscuro), `astreo-glyph-small.svg` (≤ 32 px), `astreo-glyph-dark.svg` (sobre claro) y `astreo-glyph-same.svg` (sin cambios);
  - `astreo-wordmark.svg` y `astreo-wordmark-dark.svg`, el logotipo con las letras convertidas en trazos.

### Movimiento
- **Eclipse de bienvenida:**
  1. El disco de antes aparece en 0,8 s con salida suave.
  2. A los 0,5 s, el de ahora se desliza desde la posición coincidente hasta la suya en 1,7 s con `cubic-bezier(.16,1,.3,1)`.
  3. A los 1,4 s, el titular sube 8 px mientras aparece, en 0,9 s con la misma curva.

  La animación cuenta la historia del producto: primero no hay cambios y luego asoma lo nuevo.
- **Punto vivo:** un anillo verde que crece de 0 a 6–7 px y se apaga, en 1,6 s con salida suave y en bucle.
- **Píldoras y menú** (propuesta): entran en 160 ms con subida de 4 px y aparición, y salen en 120 ms solo con desvanecido.
- **Reducir movimiento:** si está activado en macOS, el Eclipse aparece ya en su sitio y los puntos vivos son fijos.

## Do's and Don'ts

### Do:
- **Do** usar el cristal (`window` sobre `Blurred`, y `pane` encima) en toda ventana nueva, incluidas preferencias y diálogos propios.
- **Do** enseñar la tecla de cada acción visible con el componente de tecla. Astreo se usa con el teclado.
- **Do** nombrar al agente siempre con su marca real delante: "Claude Code", "Codex".
- **Do** usar mono para todo lo que se copia o se compara: símbolos, rutas, hashes y `+N −N`.
- **Do** escribir la interfaz en español, en frases cortas, sin exclamaciones y diciendo qué pasa y qué puedes hacer ("Nada sin commitear en main").
- **Do** usar los archivos de `assets/brand/` para el logo; el logotipo es un SVG con trazos, no texto.

### Don't:
- **Don't** usar fondos opacos de estilo Linear en ventanas o paneles (D37).
- **Don't** llamar a `.backdrop_blur` fuera de `src/ui/glass.rs`, ni dar cristal a otras superficies que no sean el menú y las dos píldoras (D45).
- **Don't** añadir tema claro (D16).
- **Don't** mover el scroll, cambiar el archivo abierto ni reordenar el lateral cuando llega un cambio en vivo (D20, D41).
- **Don't** usar el lavanda para algo del agente ni los colores del diff para algo que no sea el diff.
- **Don't** mostrar porcentajes de parecido. Se dice "repetida en" (D33).
- **Don't** usar un icono genérico de terminal o de robot para un agente (D39).
- **Don't** parecer una app de agentes con barra lateral, chat y paneles. El código ocupa la ventana (D18).
- **Don't** usar Geist fuera del logotipo, ni tamaños por debajo de 10,5 px.
