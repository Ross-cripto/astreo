# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

App de escritorio para macOS dibujada con GPUI: interfaz propia, sin controles de AppKit. Sigue las convenciones de ventana de macOS (semáforo, menú, atajos con Cmd, vibrancy). Por ahora solo macOS y solo tema oscuro.

## Users

Desarrolladores que trabajan con agentes de código (Claude Code, Codex y similares) y cuyo trabajo diario pasa de escribir código a revisarlo. Abren Astreo cuando un agente termina, o mientras sigue escribiendo, en el worktree donde trabaja, para entender qué cambió y juzgarlo. Es un producto para otros desde el principio.

## Product Purpose

Astreo sirve para revisar código, no para escribirlo. Lo hace leyendo el código mismo, sin paneles alrededor.

- **Vista fantasma:** lees el código tal como quedó, dividido en secciones. Lo que había antes flota tenue solo donde cambió.
- **Mapa del cambio:** muestra la forma del cambio (qué símbolos se tocaron y cómo se relacionan) y te acercas hasta el código.

Alrededor de esas dos vistas hay herramientas para el humano:
- notas que vuelven al agente;
- un modo de "una decisión a la vez";
- el porqué del agente junto a cada cambio;
- más adelante, una lente que encuentra patrones (duplicados, búsqueda estructural, puntos calientes) y análisis con IA.

Funciona si revisar el trabajo de un agente en Astreo es más rápido y más claro que en un editor, en GitHub o en una app de agentes.

## Positioning

Los editores (Cursor, Zed) están hechos para escribir. Las apps de agentes (T3 Code, cmux, Orca, Monocode) están hechas para manejar agentes y comparten la misma carcasa: barra lateral, paneles y diff rojo y verde. Astreo rechaza esa carcasa. Muestra solo el código, legible para un humano: el código final con su pasado en fantasma, en secciones, y el mapa del cambio para alejarse.

## Operating Context

El revisor alterna entre la app de su agente y Astreo. El lugar siempre es un worktree; el checkout principal también cuenta como uno. Por defecto ves lo que está sin commitear. Comparar con la rama base, con otro worktree o con otro commit existe, pero discreto. Los agentes pueden seguir escribiendo mientras revisas, y Astreo se actualiza en vivo sin moverte. Las notas vuelven al agente como archivo en el worktree, como prompt copiado o como `.md`. Se abre con "Abrir carpeta", con la lista de recientes o con `astreo .` desde la terminal.

## Capabilities and Constraints

- Solo lectura: no edita código.
- Fuentes de cambios: sin commitear (por defecto), toda la rama frente a la base, otro worktree, otra rama o commit. Los PRs de GitHub quedan fuera por ahora.
- Se construye en fases:
  - F1 Leer
  - F2 Entender
  - F3 Lente
  - F4 IA
- Stack: Rust (edición 2024) con la versión más reciente de GPUI (`gpui-pre` 0.3.x).
- Open source y gratis, con licencia GPL-3.0.
- Se distribuye como DMG firmado y notarizado, con cask de Homebrew y actualizaciones automáticas desde GitHub Releases.
- Privacidad: no envía nada a internet salvo la comprobación de actualizaciones. Sin telemetría ni informes de error.

## Brand Commitments

- Minimalista, premium y centrada en el código: quitar información antes que añadirla.
- La experiencia de usuario es lo más importante del producto.
- Monocode es solo inspiración visual de acabado (oscuro, sobrio, cuidado); no se reutiliza su código. Astreo no debe parecer una app de agentes con barra lateral y paneles.

## Evidence on Hand

- No hay usuarios, métricas, testimonios ni benchmarks: no se inventan.
- El lienzo de diseño con las exploraciones es privado: https://claude.ai/artifact/JmzCQvopvbdd8cyDkrPXBX

## Product Principles

- Revisar, no editar: cada píxel sirve para entender y juzgar.
- Solo el código: lo demás aparece cuando hace falta.
- Legible para un humano: el código final primero, el pasado como fantasma, el cambio en secciones y el mapa para la forma.
- Nunca mover al revisor: lo vivo se actualiza sin robar el sitio.
- El ciclo vuelve al agente: lo que observas se convierte en instrucciones.
