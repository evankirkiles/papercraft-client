//! The lines of the print layout, as geometry rather than pixels.
//!
//! Everything on a printed page that a person acts on is a line: the cut they
//! follow with a knife, the fold they score, the tab they glue. This emits those
//! in page space, classified, for a backend that wants them as geometry.
//!
//! **Not currently wired into the PDF export.** Printing rasterizes its lines
//! straight from the cutting viewport, which keeps the page identical to the
//! screen and keeps the depth test that tucks a tab under an overlapping face —
//! see [`crate::print`] and `docs/cut-contours.md`. What this module exists for
//! is the machine-readable cut geometry planned there: the fold classification
//! below becomes the score layer, while cuts and tab outlines are superseded by
//! contour extraction, which resolves overlap as geometry instead of by drawing
//! order.
//!
//! The classification mirrors `lines.wgsl`'s `_fold_visible` and the tab outline
//! in `flaps.wgsl`: the same taxonomy the cutting viewport draws, so the page and
//! the screen agree about what every line *is*.

use cgmath::{Point2, Point3, Transform};

use crate::{
    id::{EdgeId, FaceId, LoopId},
    measures::Rect,
    mesh::{edge::FLAT_EDGE_ANGLE_EPSILON, Mesh},
    State,
};

/// What a line on the page means.
///
/// The declaration order is **paint order**, back to front, and `Ord` follows it,
/// mirroring `pp_draw`'s `DepthClass`: tab outlines behind, then folds, then the
/// silhouette on top.
///
/// That ordering is only half the story, which is why these segments don't drive
/// the printout. `DepthClass` puts tab outlines beneath the *faces* too, and no
/// amount of paint ordering expresses that: a tab covered by an overlapping face
/// has to be *clipped*, not merely drawn early. Contour extraction is what
/// resolves it, by making the covered shape fall out of a union — see
/// `docs/cut-contours.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineKind {
    /// One of the three free sides of a glue tab. Behind every face.
    FlapOutline,
    /// A convex fold: the paper folds away from you.
    Mountain,
    /// A concave fold: the paper folds toward you.
    Valley,
    /// The mesh's own boundary, on a model that isn't closed. Also a cut, but it
    /// was never joined to anything, so it reads as the sheet's outline.
    Border,
    /// A cut edge on the piece's silhouette: cut here, and the piece comes free.
    Cut,
}

impl LineKind {
    /// Whether this is part of a piece's outline rather than something scored
    /// inside it — the lines a cutting machine would follow.
    ///
    /// Tab outlines count: three of a tab's four sides are cut, and only its base
    /// is folded.
    pub fn is_silhouette(&self) -> bool {
        matches!(self, Self::Cut | Self::Border | Self::FlapOutline)
    }
}

/// A single straight stroke on one page.
///
/// Endpoints are in **page space**: centimeters from the page's top-left corner,
/// x right and y *down*, matching how a sheet is read rather than how the
/// document's world axes run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageLine {
    pub from: Point2<f32>,
    pub to: Point2<f32>,
    pub kind: LineKind,
}

/// Every line of every piece that falls on `page`, clipped to it.
///
/// `page` is a world-space rect as returned by [`crate::print::Page::world_rect`],
/// so its `y` is the sheet's *top* edge. `fold_lines` mirrors
/// `ThemeSizes::fold_lines`: with it off, only silhouettes and tabs are emitted.
///
/// Lines are grouped by [`LineKind`] in the output, so a backend can set its
/// stroke state once per kind instead of once per segment.
pub fn page_lines(state: &State, page: Rect<f32>, fold_lines: bool) -> Vec<PageLine> {
    let mut lines = Vec::new();
    for mesh in state.meshes.values() {
        for root in mesh.iter_pieces() {
            piece_lines(mesh, *root, fold_lines, &mut lines);
        }
    }
    // Clip after collecting in world space: a piece can straddle a sheet edge,
    // and the raster layer stops at the page boundary, so the strokes have to as
    // well or they would run out onto the next sheet.
    let mut clipped: Vec<PageLine> = lines
        .into_iter()
        .filter_map(|(from, to, kind)| {
            let (from, to) =
                clip_to_page(to_page_space(from, &page), to_page_space(to, &page), &page)?;
            Some(PageLine { from, to, kind })
        })
        .collect();
    clipped.sort_by_key(|line| line.kind);
    clipped
}

/// World-space endpoints of one piece's lines, tagged by kind.
type WorldLine = (Point3<f32>, Point3<f32>, LineKind);

/// Appends every line of the piece rooted at `root`.
fn piece_lines(mesh: &Mesh, root: FaceId, fold_lines: bool, out: &mut Vec<WorldLine>) {
    let Some(piece) = mesh.pieces.get(&root) else { return };
    let walker = mesh.iter_piece_faces_unfolded(root);
    let t = walker.t;

    // An interior edge is reached from both of its faces, at identical unfolded
    // positions, so it would otherwise be stroked twice. A *cut* edge is only
    // ever reached once per piece — its other face belongs to another piece,
    // which draws its own copy of the seam somewhere else on the page.
    let mut seen: Vec<EdgeId> = Vec::new();

    for face in walker {
        for l_id in mesh.iter_face_loops(face.f) {
            let place = |p: Point3<f32>| piece.transform.transform_point(p);
            let unfolded = |v| {
                place(face.affine.transform_point(Point3::new(0.0, 0.0, 0.0) + mesh.vert_pos(v)))
            };

            if let Some(corners) = mesh.piece_flap_corners(l_id, face.affine, t) {
                // Side 0 of the trapezoid is its base, which lies on the seam the
                // tab folds along; that line is emitted as a fold below. Only the
                // three free sides are the tab's own outline.
                for side in 1..4 {
                    out.push((
                        place(corners[side]),
                        place(corners[(side + 1) % 4]),
                        LineKind::FlapOutline,
                    ));
                }
            }

            let e_id = mesh[l_id].e;
            let Some(kind) = line_kind(mesh, l_id) else { continue };
            if !fold_lines && !kind.is_silhouette() {
                continue;
            }
            if seen.contains(&e_id) {
                continue;
            }
            seen.push(e_id);

            let e = mesh[e_id];
            out.push((unfolded(e.v[0]), unfolded(e.v[1]), kind));
        }
    }
}

/// What kind of line loop `l_id` contributes, or `None` if it draws nothing.
///
/// Mirrors `_fold_visible` in `lines.wgsl`. The subtle case is a cut that carries
/// a flap: that seam is the line the tab folds along, so it is a fold rather than
/// part of the silhouette — cutting it would take the tab off with it.
fn line_kind(mesh: &Mesh, l_id: LoopId) -> Option<LineKind> {
    let l = mesh[l_id];
    if l.radial_next == l_id {
        return Some(LineKind::Border);
    }
    let is_cut = mesh.edge_is_cut(&l.e);
    if is_cut && !mesh.loop_has_flap(l_id) {
        return Some(LineKind::Cut);
    }
    // Flat enough to not be a fold at all, so nothing is drawn.
    let angle = mesh.edge_fold_angle(l.e)?;
    if angle.abs() < FLAT_EDGE_ANGLE_EPSILON {
        return None;
    }
    Some(if angle > 0.0 { LineKind::Mountain } else { LineKind::Valley })
}

/// Converts a world point to page space: centimeters from the sheet's top-left,
/// y running down the sheet.
///
/// The printable quadrant runs right and *down* from the world origin, so a
/// page's `y` is its top edge and points on it have smaller `y` than that.
fn to_page_space(p: Point3<f32>, page: &Rect<f32>) -> Point2<f32> {
    Point2::new(p.x - page.x, page.y - p.y)
}

/// Trims a segment to the page, or drops it if it misses entirely.
///
/// Liang-Barsky: walk the four edges narrowing the parameter range the segment
/// is inside for, and bail as soon as that range is empty.
fn clip_to_page(
    from: Point2<f32>,
    to: Point2<f32>,
    page: &Rect<f32>,
) -> Option<(Point2<f32>, Point2<f32>)> {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let (mut t0, mut t1) = (0.0f32, 1.0f32);

    // Each edge as "the segment must stay on the inside of it": `p` is how fast
    // the segment approaches the edge, `q` how far inside it starts.
    for (p, q) in
        [(-dx, from.x), (dx, page.width - from.x), (-dy, from.y), (dy, page.height - from.y)]
    {
        if p == 0.0 {
            // Parallel to this edge, so it can only be wholly in or wholly out.
            if q < 0.0 {
                return None;
            }
            continue;
        }
        let r = q / p;
        if p < 0.0 {
            if r > t1 {
                return None;
            }
            t0 = t0.max(r);
        } else {
            if r < t0 {
                return None;
            }
            t1 = t1.min(r);
        }
    }
    if t0 > t1 {
        return None;
    }
    Some((
        Point2::new(from.x + t0 * dx, from.y + t0 * dy),
        Point2::new(from.x + t1 * dx, from.y + t1 * dy),
    ))
}

#[cfg(test)]
mod tests {
    use cgmath::{Matrix4, Vector3};

    use super::*;
    use crate::{
        id::{Id, VertexId},
        mesh::cut::{CutUpdate, FlapPosition},
        print::{Page, PageSize},
        MeshId,
    };

    /// The four edges ringing the cube's bottom quad. Cutting them frees the
    /// bottom's two triangles into a piece — the same fixture the flap tests use.
    const BOTTOM_RING: [(usize, usize); 4] = [(0, 1), (1, 2), (2, 3), (3, 0)];

    /// A cube with its bottom quad cut free into a single piece, laid flat and
    /// parked well inside the first sheet.
    ///
    /// The move matters: pages tile the quadrant `x >= 0, y <= 0`, so a piece left
    /// at the world origin straddles the first sheet's top-left corner and most of
    /// it clips away.
    fn cube_with_one_piece() -> (State, MeshId, FaceId) {
        let mut state = State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        for (a, b) in BOTTOM_RING {
            let e_id = state.meshes[m_id]
                .query_edge(VertexId::from_usize(a), VertexId::from_usize(b))
                .unwrap();
            state.meshes[m_id].make_cut(e_id, CutUpdate::PiecesAndFlaps);
        }
        // Flatten the piece: `t = 1` is what printing always sees.
        let root = *state.meshes[m_id].iter_pieces().next().unwrap();
        state.meshes[m_id].pieces.get_mut(&root).unwrap().t = 1.0;
        move_piece(&mut state, m_id, root, 5.0, -5.0);
        (state, m_id, root)
    }

    fn move_piece(state: &mut State, m_id: MeshId, root: FaceId, x: f32, y: f32) {
        state.meshes[m_id]
            .transform_piece(&root, Matrix4::from_translation(Vector3::new(x, y, 0.0)));
    }

    /// The sheet at grid cell `(col, row)`.
    fn page(col: f32, row: f32) -> Rect<f32> {
        Page { pos: Point2::new(col, row), label: None }.world_rect(&PageSize::A4)
    }

    fn of_kind(lines: &[PageLine], kind: LineKind) -> usize {
        lines.iter().filter(|l| l.kind == kind).count()
    }

    /// Bares one seam of its tab, so that seam becomes silhouette.
    fn bare_a_seam(state: &mut State, m_id: MeshId) {
        let e_id = state.meshes[m_id]
            .query_edge(VertexId::from_usize(0), VertexId::from_usize(1))
            .unwrap();
        state.meshes[m_id].set_cut_flap(e_id, FlapPosition::None);
    }

    /// The bottom quad's four edges each carry a glue tab, and a cut that carries
    /// a tab is the line that tab folds along — cutting it would take the tab off
    /// with it. So the ring prints as folds plus three tab sides each, and the
    /// coplanar diagonal between the quad's two triangles prints as nothing.
    #[test]
    fn a_seam_carrying_a_tab_prints_as_a_fold_not_a_cut() {
        let (state, ..) = cube_with_one_piece();
        let lines = page_lines(&state, page(0.0, 0.0), true);
        assert_eq!(
            of_kind(&lines, LineKind::Mountain),
            4,
            "the cube's four right-angle ring folds"
        );
        assert_eq!(of_kind(&lines, LineKind::FlapOutline), 12, "three free sides per tab");
        assert_eq!(of_kind(&lines, LineKind::Cut), 0, "a seam with a tab is not a silhouette");
        assert_eq!(
            of_kind(&lines, LineKind::Valley),
            0,
            "a cube unfolds outward, so every fold on it reads convex"
        );
        assert_eq!(lines.len(), 16, "and nothing else, the flat diagonal included: {lines:#?}");
    }

    /// Take the tab off one seam and that seam becomes silhouette instead: with
    /// nothing to fold onto, it is simply where you cut.
    #[test]
    fn a_seam_without_a_tab_is_silhouette() {
        let (mut state, m_id, _) = cube_with_one_piece();
        bare_a_seam(&mut state, m_id);

        let lines = page_lines(&state, page(0.0, 0.0), true);
        assert_eq!(of_kind(&lines, LineKind::Cut), 1, "the bared seam");
        assert_eq!(of_kind(&lines, LineKind::Mountain), 3, "the three that kept their tabs");
        assert_eq!(of_kind(&lines, LineKind::FlapOutline), 9, "and one tab fewer");
    }

    /// Every interior edge is reached from both of its faces. Stroking it twice
    /// would double the ink and, for a dashed fold, fill in the gaps and print a
    /// solid line.
    #[test]
    fn an_interior_edge_is_stroked_only_once() {
        let (state, ..) = cube_with_one_piece();
        let lines = page_lines(&state, page(0.0, 0.0), true);
        for (i, a) in lines.iter().enumerate() {
            for b in &lines[i + 1..] {
                let same = (a.from == b.from && a.to == b.to) || (a.from == b.to && a.to == b.from);
                assert!(!same, "the same segment was emitted twice: {a:?}");
            }
        }
    }

    /// Turning fold lines off leaves the silhouette and the tabs, which are what
    /// you still have to cut and glue.
    #[test]
    fn switching_fold_lines_off_keeps_the_silhouette() {
        let (mut state, m_id, _) = cube_with_one_piece();
        bare_a_seam(&mut state, m_id);

        let lines = page_lines(&state, page(0.0, 0.0), false);
        assert_eq!(of_kind(&lines, LineKind::Cut), 1, "the cut must survive");
        assert_eq!(of_kind(&lines, LineKind::FlapOutline), 9, "and so must the tabs");
        assert!(
            !lines.iter().any(|l| matches!(l.kind, LineKind::Mountain | LineKind::Valley)),
            "but no folds"
        );
    }

    /// A piece straddling a sheet boundary is printed on both sheets, each getting
    /// its own side of the split. Dropping it from either would tear the model.
    #[test]
    fn a_piece_spanning_two_sheets_prints_on_both() {
        let (mut state, m_id, root) = cube_with_one_piece();
        // Undo the fixture's placement, then straddle the first column's right
        // edge: the cube is 1cm across, so half of it lands on each sheet.
        move_piece(&mut state, m_id, root, -5.0, 5.0);
        let seam = PageSize::A4.dimensions().width;
        move_piece(&mut state, m_id, root, seam - 0.5, -5.0);

        let left = page_lines(&state, page(0.0, 0.0), true);
        let right = page_lines(&state, page(1.0, 0.0), true);
        assert!(!left.is_empty(), "the left sheet should get the piece's left half");
        assert!(!right.is_empty(), "and the right sheet its right half");

        // Neither sheet may carry ink past its own edge.
        for (name, lines) in [("left", &left), ("right", &right)] {
            for line in lines.iter() {
                for p in [line.from, line.to] {
                    assert!(
                        p.x >= -1e-3 && p.x <= seam + 1e-3,
                        "{name} sheet has a point off it at {p:?}"
                    );
                }
            }
        }
    }

    /// Page space reads down from each sheet's own top-left corner, so a piece on
    /// the second row comes out near the top of its own page rather than at its
    /// distance from the world origin.
    #[test]
    fn page_space_is_measured_from_the_sheets_own_corner() {
        let second = page(1.0, 1.0);
        // A point one centimeter in from that sheet's top-left corner.
        let world = Point3::new(second.x + 1.0, second.y - 1.0, 0.0);
        let p = to_page_space(world, &second);
        assert!((p.x - 1.0).abs() < 1e-4 && (p.y - 1.0).abs() < 1e-4, "got {p:?}");
    }

    /// A line running off the sheet is trimmed at the edge, not dropped and not
    /// left to run onto the next page.
    #[test]
    fn a_line_crossing_the_page_edge_is_trimmed_to_it() {
        let sheet = Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        let clipped = clip_to_page(Point2::new(5.0, 5.0), Point2::new(15.0, 5.0), &sheet).unwrap();
        assert_eq!(clipped.0, Point2::new(5.0, 5.0));
        assert_eq!(clipped.1, Point2::new(10.0, 5.0), "should stop at the right edge");
    }

    /// A line spanning the whole sheet is clipped at both ends at once.
    #[test]
    fn a_line_spanning_the_page_is_trimmed_at_both_ends() {
        let sheet = Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        let clipped = clip_to_page(Point2::new(-5.0, 5.0), Point2::new(15.0, 5.0), &sheet).unwrap();
        assert_eq!(clipped.0, Point2::new(0.0, 5.0));
        assert_eq!(clipped.1, Point2::new(10.0, 5.0));
    }

    /// A line entirely off the sheet contributes nothing to it — including one
    /// running parallel to an edge, which the parametric clip has to special-case.
    #[test]
    fn a_line_off_the_page_is_dropped() {
        let sheet = Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        assert!(clip_to_page(Point2::new(-5.0, 5.0), Point2::new(-1.0, 5.0), &sheet).is_none());
        assert!(clip_to_page(Point2::new(2.0, 20.0), Point2::new(8.0, 20.0), &sheet).is_none());
        assert!(clip_to_page(Point2::new(-2.0, 0.0), Point2::new(-2.0, 10.0), &sheet).is_none());
    }

    /// An empty document prints an empty page rather than failing.
    #[test]
    fn a_document_with_no_pieces_has_no_lines() {
        let state = State::default();
        assert!(page_lines(&state, page(0.0, 0.0), true).is_empty());
    }

    /// Lines arrive grouped by kind so a backend sets its stroke state once per
    /// group instead of once per segment.
    #[test]
    fn lines_come_back_grouped_by_kind() {
        let (state, ..) = cube_with_one_piece();
        let lines = page_lines(&state, page(0.0, 0.0), true);
        let kinds: Vec<LineKind> = lines.iter().map(|l| l.kind).collect();
        let mut sorted = kinds.clone();
        sorted.sort();
        assert_eq!(kinds, sorted, "kinds should already be in order");
    }
}
