# TODO

Known gaps and planned work, newest first. Each entry links to a plan when there
is one.

## Cut contours for the print/cut path

**Status:** planned — see [cut-contours.md](cut-contours.md)

The PDF export prints correctly but carries no machine-readable geometry: a page
is a single rasterized image of the sheet. A cutting machine needs the cut
outlines as closed contours.

That is the same work that fixes tab occlusion properly. When an unfolded piece
overlaps *itself* — a piece is a tree of faces, and flattening a tree can fold it
back over its own body — a tab's outward direction points into its own piece, and
only a sliver emerges past the silhouette. That sliver is the real cut shape: you
cut the outline, notch between the tabs, and the small tabs still glue. Getting it
exactly needs a 2D boolean, not mesh topology.

[`pp_core::print::vector`](../crates/pp_core/src/print/vector.rs) is the starting
point and is **deliberately not wired into anything today**. Its fold
classification becomes the score layer; its cut and tab-outline segments are
superseded by contours.

## Texture bleed onto tabs

**Status:** wanted, not planned

Extend each face's texture past its own triangle and onto the tabs, so a glued
seam doesn't show a white sliver where printer registration drifts. Tabs would
then carry texture rather than flat white paper, which is part of why print keeps
its lines rasterized (below).

## Smaller things

- **Print lines are rasterized, deliberately.** A page is the cutting viewport's
  own render, so the two cannot drift and the depth test keeps a tab tucked under
  the face that covers it. Vector strokes would print crisper — they would let
  line resolution be independent of the 300 DPI raster — but reproducing the depth
  test in PDF needs the page image to be transparent off-face, i.e. a soft mask,
  which is a *second* full-page image for lines the raster already contains. The
  taxonomy would also live twice, once in `lines.wgsl` and once in the writer,
  with nothing to catch it drifting. If print line quality becomes the complaint,
  raising the print DPI is the first lever: A4 at 600 DPI is ~139MB of readback,
  and 1200 DPI exceeds the 8192 texture limit.
- `RENDER_FOLD_LINES` became `ThemeSizes::fold_lines`
  ([`preferences::theme`](../crates/pp_editor/src/preferences/theme.rs)) but has
  no settings UI yet, so it can only be changed in code.
- **A print run stalls while its tab is hidden.** It is driven by
  `requestAnimationFrame`, which browsers suspend in background tabs, so switching
  away mid-print pauses it until you switch back. Inherent to the frame-driven
  design in [`print`](../crates/pp_client/src/print.rs), which exists because
  `map_async` can only complete on the browser's event loop.
- `TextBox` and `ImageBox` ([`print`](../crates/pp_core/src/print/)) are stubs:
  they live in `State` but nothing renders them or associates them with a page.
- Every shader redeclares `struct ThemeSizes`, so adding a theme size means
  editing all of `engines/**/shaders/`. Worth generating or sharing.
- `pp_draw` and `pp_client` have no tests, so nothing covers `PrintTarget`,
  `print_page`/`print_poll` or `PrintJob`. The geometry they consume is covered in
  `pp_core`, and the PDF writer in `pp_save`.
