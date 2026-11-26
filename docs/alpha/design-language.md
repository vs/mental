# Design language

Every visual this project makes follows it: diagrams and charts in the docs, images, and the UI. It guides; it isn't a wall. A visual may step outside it for a reason the change states (a chart that needs more series colors, a one-off illustration). What it must not do is drift: a near-copy of a palette color, a second accent, a typeface nobody chose. Something the project will keep using goes into the design language first (`/alpha:update design language`).

The reviewer checks changed visuals against this page, and is shown each color outside the palette in diagrams, SVG and stylesheets to judge: a deliberate exception passes, drift is a finding. `[visuals]` in [principles.toml](principles.toml) lists the files that count as visuals, and the third-party and generated ones that don't.

## Character

Warm, quiet and precise: warm paper-white neutrals, one copper accent, hairline borders, depth only for floating panels; dense, tool-like layouts with monospace for anything the machine produced (IDs, values, traces).

## Colors

| Name | Value | Use |
|---|---|---|
| background | `#fdfcfa` | page and diagram background |
| surface | `#f3f2f0` | panels and groups, such as a system boundary in a diagram |
| raised | `#ffffff` | boxes and cards: components, records, steps |
| inset | `#f7f6f4` | code blocks and terminal output |
| text | `#262521` | titles and body text |
| muted-text | `#696960` | secondary labels, captions, annotations |
| line | `#d8d5d0` | hairline borders, dividers and arrows |
| accent | `#b35017` | the subject of a diagram, the path being explained, primary actions |
| accent-fill | `#f6ebe3` | the fill behind an accented element |
| accent-text | `#9a4310` | accented text and links |
| success | `#517734` | fixed, passed, matched |
| warning | `#886914` | a preemption, an assumption, a heuristic match |
| danger | `#b7493f` | a violation, an illegal write, a failed check |
| info | `#2c6ebd` | neutral highlights and notes |
| chart-series | `oklch(68% .12 230), oklch(68% .12 145), oklch(70% .13 95), oklch(65% .14 45)` | chart series 1 to 4, in that order |

## Typography

Fixel Display SemiBold for headings, letter-spacing -0.025em; Fixel Text Regular for prose; Cascadia Mono Medium for code, IDs, labels and values; fallbacks system-ui for sans and SF Mono, ui-monospace for mono

## Shape

6 px corners on small elements, 9 px on controls, 12 px on cards and panels, 16 px on large frames; hairline low-contrast borders; flat by default, soft layered shadows only on floating panels and popovers; small uppercase labels used sparingly; 150 ms transitions

## Icons and imagery

Remix Icon line icons at 12 to 24 px

## Diagrams and charts

SVG source committed in docs/images/, light theme: groups are surface panels, components are raised boxes with a line-colored hairline, flows are muted arrows, the subject is in the accent; status colors only for findings (danger a violation, warning a preemption or assumption, success fixed); titles in Fixel, labels in Cascadia Mono; every image has alt text
