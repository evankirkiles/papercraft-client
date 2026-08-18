# Plan: cut contours

**Status:** planned, not started.
**Prerequisite:** the PDF export (landed).

## Why

The PDF export prints correctly — each page is the cutting viewport's own render,
placed as an image — but it carries no geometry a machine can act on.

An earlier revision stroked the lines into the PDF as vector, from
`pp_core::print::vector::page_lines`. That was **reverted** to rasterizing, for
two reasons worth recording so they are not re-litigated:

- **PDF has no depth test.** The viewport resolves overlap with `DepthClass`,
  which places `FlapOutline` (1) *below* `Surface` (2), so a tab under a face is
  hidden ([`engines::ink`](../crates/pp_draw/src/engines/ink/mod.rs)). Paint order
  alone cannot express that — a covered tab has to be *clipped*, not drawn early —
  and the workaround (a transparent-off-face page image, i.e. a soft mask) is a
  second full-page image for lines the raster already contains. Planned texture
  bleed onto tabs makes that payload worse.
- **The taxonomy would live twice**, once in `lines.wgsl` and once in the writer,
  with nothing to catch it drifting. It already did drift once: the writer reached
  for the theme's `edge_cut` and `line_width_thick`, which belong to the *folding*
  viewport's `lines_cut` pipeline, and printed red cut lines that appear nowhere on
  screen.

So the remaining gap is not how the page *looks* — it is that a cutter fed a
raster image, or fed disconnected segments, has nothing to follow. A machine
lifts and drops the blade at every vertex and compounds registration error around
the piece. It wants **closed, oriented contours**, which is a different
computation from anything the renderer does.

### The case that also matters

A piece is a spanning tree of faces — `assert_face_can_make_piece`
([`mesh::piece`](../crates/pp_core/src/mesh/piece.rs)) rejects cycles — so
flattening it can fold the piece back over its own body. Where that happens, a
tab's "outward" direction points *into* its own piece, and only a sliver of the
trapezoid emerges past the silhouette.

That sliver is not a defect to be designed away. It is a standard papercraft
pattern: cut the outline, notch out between the tabs, and the small tabs glue
fine. So the printed and cut geometry must reproduce it exactly.

Note this is a *geometric* fact, not a topological one, so no amount of mesh
walking reaches it. It needs a real 2D boolean.

## The approach

Per piece, take the boundary of

```
R  =  union(unfolded face triangles)  ∪  union(tab trapezoids)
```

under a **non-zero** fill rule. Everything falls out of that one operation, with
no special cases:

| behaviour | why it works |
|---|---|
| the occluded sliver | the covering faces are already in `R`, so a tab contributes only the part past the silhouette |
| notches between tabs | two tabs on adjacent edges meet only at their shared base vertex, so the boundary runs out around one, back to the base, then out around the next |
| piece self-overlap | a folded-back piece is just a self-overlapping triangle set; non-zero winding collapses it to the region actually covered |
| holes | a piece wrapping into a ring returns an outer contour plus hole contours |
| fold lines excluded | a tab's base edge is interior to `R`, so it never appears in the cut boundary — it belongs to the score layer |

### Validated

A throwaway probe against `i_overlay` confirmed all of it on real `pp_core`
geometry before this plan was written:

- **Real piece** (cube's bottom quad cut free, 2 face triangles + 4 tabs) → one
  shape, an 8-point octagon. Adjacent tabs' 45° sides are collinear off each
  corner, so they abut exactly and the redundant vertices simplify away.
- **Occluded tab** (trapezoid with top at y=0.6 under a face covering to y=0.45)
  → one shape whose tallest point is y=0.600. The sliver survives exactly.
- **Two tabs on one edge with a gap** → 12-point contour that returns to the base
  edge between them: the notch.

## Steps

### 0. What already exists

[`pp_core::print::vector`](../crates/pp_core/src/print/vector.rs) is retained,
unwired, as the starting point: the piece traversal, the fold classification
(mirroring `_fold_visible`), page-space conversion and clipping are all written
and tested. Its `Mountain` / `Valley` output becomes the score layer directly; its
`Cut`, `Border` and `FlapOutline` output is what contours replace.

### 1. Add `i_overlay` to `pp_core`

Pure Rust, builds clean for `wasm32-unknown-unknown`, four transitive deps
(`i_float`, `i_key_sort`, `i_shape`, `i_tree`) with `libm` the only leaf.

`SimplifyShape::simplify_shape_as::<i64>(FillRule::NonZero)` over a contour
collection is the union; it returns `Shapes<P>` — outer contours first, then
holes, winding documented. Use the `i64` engine: the internal fixed-point snap
wants headroom, and page coordinates are centimeters with features from ~0.01cm
(a stroke width) to ~30cm (a sheet).

### 2. `pp_core::print::contour`

A new sibling to [`print::vector`](../crates/pp_core/src/print/vector.rs):

```rust
/// A piece's cut outline: one outer contour, plus one per hole.
pub struct PieceContour {
    pub outer: Vec<Point2<f32>>,
    pub holes: Vec<Vec<Point2<f32>>>,
}

/// The cut outline of every piece overlapping `page`, in page space.
pub fn page_contours(state: &State, page: Rect<f32>) -> Vec<PieceContour>;
```

Reuse the traversal `page_lines` already uses — `iter_pieces` →
`iter_piece_faces_unfolded` → `iter_face_loops`, with
`piece.transform * face.affine` for position — and `Mesh::piece_flap_corners`
([`mesh::flap`](../crates/pp_core/src/mesh/flap.rs)) for the tabs. Feed the
triangles and trapezoids in as contours, union, convert to page space, clip.

Clipping needs to become polygon-vs-rect rather than the segment clip
`page_lines` uses; `i_overlay` can do it as an intersection with the page rect,
which also keeps contours closed across the sheet boundary.

### 3. Split `page_lines` into score lines only

Once contours own the cut geometry, `page_lines` should emit only what gets
*scored*: `Mountain` and `Valley`. `Cut`, `Border` and `FlapOutline` become
contour output, and `LineKind::is_silhouette` goes away.

Keep the fold classification exactly as it is — it mirrors `lines.wgsl`'s
`_fold_visible` and is already tested.

### 4. `pp_save::pdf`: carry the contours

The writer is currently image-only. Add the cut geometry *alongside* the page
image rather than in place of it:

- One `m`/`l`…`h` subpath per contour, so a piece is a single path object.
- Put them in an optional content group named "Cut", defaulting to **off**, so
  printing is unchanged and a cutter or a designer can isolate them. This is the
  key point: the contours are data in the file, not the visible ink, so the page
  keeps matching the viewport exactly.
- Occlusion needs no handling — the contour already excludes what a face covers.

An SVG export (step 5) may matter more to users than the PDF layer; do whichever
the actual cutting workflow asks for first.

### 5. SVG export (optional, nearly free)

With contours in page space, an SVG writer is the same data through a different
serializer: cut contours in one group, score lines in another. This is what the
cutting-machine and laser users will actually ask for, and unlike the PDF layer it
needs no compromise with the printed page.

## Out of scope

**Cross-piece overlap.** A per-piece union handles self-overlap. If piece A's tab
is covered by piece *B*'s face, matching the viewport would mean subtracting other
pieces' faces too — but a layout with overlapping pieces cannot be cut from one
sheet either way. Treat it as layout validation (pairwise contour intersection,
surfaced as a warning) rather than folding it into the contour. Separate work.

**Viewport convergence.** Once contours exist, the cutting viewport could draw
them instead of depth-sorted segments, retiring the `FlapOutline` depth class and
making the tab silhouette exact rather than depth-resolved. Worth doing, but it is
a renderer change with its own risk — and note the page already matches the screen
by construction, so this buys correctness, not consistency.

## Verification

Unit tests in `pp_core`, in the repo's inline `#[cfg(test)] mod tests` style with
full-sentence names:

- A cut quad with four abutting tabs gives one closed contour and no holes.
- A tab covered by a face of its own piece keeps exactly the part that protrudes
  (assert the extreme point survives).
- Two tabs on one edge with a gap leave a notch — the contour returns to the base
  edge between them.
- A piece spanning two sheets yields a closed contour on each.
- Contour winding is consistent, so a cutter can infer inside from outside.

Then end to end: export a model with a self-overlapping piece and confirm the
contours carry the sliver and the notches, and that the `Cut` layer isolates only
them. A small example binary in `pp_save` writing an SVG of the contours is the
fastest way to eyeball this without a browser or a cutting machine.
