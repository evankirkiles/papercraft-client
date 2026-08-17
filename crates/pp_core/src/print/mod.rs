pub mod image_box;
pub mod text_box;

use std::collections::BTreeSet;

use slotmap::SlotMap;

pub use image_box::*;
pub use text_box::*;

use crate::{bounds::Aabb3, measures::Dimensions, PageId};

/// Centimeters per inch, used to express physical page sizes (traditionally
/// specified in inches) in the document's world units (1 unit = 1 cm).
pub const CM_PER_INCH: f32 = 2.54;

/// Upper bound on the auto-fitted page grid along either axis, so a runaway
/// piece transform can't make us allocate an enormous page buffer.
const MAX_PAGES_PER_AXIS: u32 = 64;

#[derive(Debug, Default, Clone)]
pub enum PageSize {
    #[default]
    A4,
    Letter,
    Custom(Dimensions<f32>),
}

impl PageSize {
    pub fn dimensions(&self) -> Dimensions<f32> {
        match self {
            PageSize::A4 => Dimensions { width: 21.0, height: 29.7 },
            PageSize::Letter => Dimensions { width: 8.5 * CM_PER_INCH, height: 11.0 * CM_PER_INCH },
            PageSize::Custom(dims) => *dims,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    /// The top-left position of this page
    pub pos: cgmath::Point2<f32>,
    /// An internal name for the page
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrintLayout {
    /// The dimensions of each page
    pub page_size: PageSize,
    /// Margins at the top left of pages
    pub page_margin_start: cgmath::Point2<f32>,
    /// Margins at the bottom right of pages
    pub page_margin_end: cgmath::Point2<f32>,

    /// Page-specific configuration
    pub pages: SlotMap<PageId, Page>,

    /// The number of page columns / rows currently laid out, derived from the
    /// pieces by [`crate::State::fit_pages_to_pieces`]. `pages` always holds
    /// exactly the `cols * rows` cells of this grid; the counts are cached
    /// here so the renderer knows the grid's extent without scanning them.
    pub cols: u32,
    pub rows: u32,

    /// Are page-level resources dirty? (e.g. we need to recreate the vbuf)
    pub elem_dirty: bool,
    /// Are setting-level resources dirty? (e.g. we need to recreate the uniform)
    pub is_dirty: bool,
}

impl Default for PrintLayout {
    fn default() -> Self {
        let mut pages = SlotMap::with_key();
        pages.insert(Page { pos: cgmath::Point2 { x: 0.0, y: 0.0 }, label: None });
        Self {
            page_size: Default::default(),
            page_margin_start: cgmath::Point2 { x: 0.5 * CM_PER_INCH, y: 0.5 * CM_PER_INCH },
            page_margin_end: cgmath::Point2 { x: 0.5 * CM_PER_INCH, y: 0.5 * CM_PER_INCH },
            pages,
            cols: 1,
            rows: 1,
            elem_dirty: true,
            is_dirty: true,
        }
    }
}

impl PrintLayout {
    /// Resizes the page grid to the smallest one covering `bounds`, the
    /// unfolded extent of the pieces sitting in the printable quadrant.
    ///
    /// Pages tile contiguously — page `(c, r)` sits at `pos = (c, r)` in page
    /// units, which the page shader multiplies straight through by the page
    /// dimensions — so there is no gutter for a piece to fall into. The grid
    /// both grows and shrinks, but never below a single page.
    pub fn fit_to_bounds(&mut self, bounds: &Aabb3) {
        let Dimensions { width, height } = self.page_size.dimensions();
        let axis = |extent: f32, page: f32| {
            ((extent.max(0.0) / page).ceil().max(1.0) as u32).min(MAX_PAGES_PER_AXIS)
        };
        let (cols, rows) = if bounds.is_empty() {
            (1, 1)
        } else {
            // The quadrant runs right and *down* from the origin, so the rows
            // are driven by how far below y=0 the pieces reach.
            (axis(bounds.max.x, width), axis(-bounds.min.y, height))
        };
        // This runs every frame: bail before touching the dirty flags when the
        // grid is already the right size, so we don't re-upload page buffers.
        if (cols, rows) == (self.cols, self.rows) {
            return;
        }

        // Reconcile in place rather than rebuilding the slotmap, so page ids
        // and labels survive a resize.
        let mut existing: BTreeSet<(u32, u32)> = BTreeSet::new();
        self.pages.retain(|_, page| {
            let cell = (page.pos.x.round().max(0.0) as u32, page.pos.y.round().max(0.0) as u32);
            // Drop pages outside the new grid, and any duplicate landing on a
            // cell we've already kept
            cell.0 < cols && cell.1 < rows && existing.insert(cell)
        });
        for r in 0..rows {
            for c in 0..cols {
                if existing.contains(&(c, r)) {
                    continue;
                }
                self.pages
                    .insert(Page { pos: cgmath::Point2::new(c as f32, r as f32), label: None });
            }
        }

        self.cols = cols;
        self.rows = rows;
        self.elem_dirty = true;
        self.is_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use super::*;

    /// A box spanning from the origin to `(x, y)` in the printable quadrant,
    /// i.e. `y` is negative.
    fn bounds(x: f32, y: f32) -> Aabb3 {
        let mut aabb = Aabb3::EMPTY;
        aabb.extend(Vector3::new(0.0, 0.0, 0.0));
        aabb.extend(Vector3::new(x, y, 0.0));
        aabb
    }

    fn cells(layout: &PrintLayout) -> BTreeSet<(u32, u32)> {
        layout.pages.values().map(|p| (p.pos.x.round() as u32, p.pos.y.round() as u32)).collect()
    }

    #[test]
    fn the_grid_covers_the_pieces_exactly() {
        let mut layout = PrintLayout::default();
        let Dimensions { width, height } = layout.page_size.dimensions();

        // A hair over one page in each direction needs a second page in each
        layout.fit_to_bounds(&bounds(width + 0.1, -height - 0.1));
        assert_eq!((layout.cols, layout.rows), (2, 2));
        assert_eq!(layout.pages.len(), 4);
        assert_eq!(cells(&layout), BTreeSet::from([(0, 0), (1, 0), (0, 1), (1, 1)]));

        // Exactly one page fits on one page
        layout.fit_to_bounds(&bounds(width, -height));
        assert_eq!((layout.cols, layout.rows), (1, 1));
        assert_eq!(cells(&layout), BTreeSet::from([(0, 0)]));
    }

    #[test]
    fn an_empty_document_keeps_a_single_page() {
        let mut layout = PrintLayout::default();
        layout.fit_to_bounds(&Aabb3::EMPTY);
        assert_eq!((layout.cols, layout.rows), (1, 1));
        assert_eq!(layout.pages.len(), 1);
    }

    /// The grid is fitted every frame, so a no-op fit must not mark the layout
    /// dirty — that would re-upload the page buffers on every single frame.
    #[test]
    fn refitting_an_unchanged_grid_is_a_no_op() {
        let mut layout = PrintLayout::default();
        let Dimensions { width, height } = layout.page_size.dimensions();
        layout.fit_to_bounds(&bounds(width * 1.5, -height * 1.5));
        let pages = layout.pages.len();
        layout.elem_dirty = false;
        layout.is_dirty = false;

        layout.fit_to_bounds(&bounds(width * 1.5, -height * 1.5));
        assert!(!layout.elem_dirty && !layout.is_dirty);
        assert_eq!(layout.pages.len(), pages);
    }

    /// Shrinking and re-growing the grid shouldn't churn the pages that stayed
    /// put: their ids and labels have to survive.
    #[test]
    fn surviving_pages_keep_their_identity() {
        let mut layout = PrintLayout::default();
        let Dimensions { width, height } = layout.page_size.dimensions();
        let first = layout.pages.keys().next().unwrap();
        layout.pages[first].label = Some("cover".to_string());

        layout.fit_to_bounds(&bounds(width * 3.0, -height * 2.0));
        assert_eq!(layout.pages.len(), 6);
        assert_eq!(layout.pages[first].label.as_deref(), Some("cover"));

        layout.fit_to_bounds(&bounds(width, -height));
        assert_eq!(layout.pages.len(), 1);
        assert_eq!(layout.pages[first].label.as_deref(), Some("cover"));
    }

    /// The end-to-end promise: a piece dragged off the first sheet is still
    /// sitting over a page afterwards, with no gap between the sheets it
    /// spans.
    #[test]
    fn a_piece_dragged_off_the_first_sheet_is_still_over_a_page() {
        use crate::id::{FaceId, Id};

        let mut state = crate::State::default();
        let m_id = state.meshes.insert(crate::mesh::Mesh::new_tri());
        let f_id = FaceId::from_usize(0);
        state.meshes[m_id].expand_piece(f_id).unwrap();
        let Dimensions { width, height } = state.printing.page_size.dimensions();

        state.meshes[m_id].transform_piece(
            &f_id,
            cgmath::Matrix4::from_translation(Vector3::new(width * 1.5, -height * 2.5, 0.0)),
        );
        state.fit_pages_to_pieces();

        let (cols, rows) = (state.printing.cols, state.printing.rows);
        let piece = state.piece_bounds_in_print_quadrant();
        assert!(
            piece.max.x <= width * cols as f32 && piece.min.y >= -height * rows as f32,
            "the {cols}x{rows} grid should cover the piece at {piece:?}"
        );
        // ...and it should be the *smallest* such grid
        assert!(piece.max.x > width * (cols - 1) as f32);
        assert!(piece.min.y < -height * (rows - 1) as f32);
        // Every cell of the grid is filled, so the sheets tile without gaps
        let expected: BTreeSet<_> =
            (0..rows).flat_map(|r| (0..cols).map(move |c| (c, r))).collect();
        assert_eq!(cells(&state.printing), expected);
    }

    #[test]
    fn a_runaway_piece_cannot_blow_up_the_grid() {
        let mut layout = PrintLayout::default();
        layout.fit_to_bounds(&bounds(1.0e9, -1.0e9));
        assert_eq!((layout.cols, layout.rows), (MAX_PAGES_PER_AXIS, MAX_PAGES_PER_AXIS));
    }
}
