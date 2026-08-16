pub mod image_box;
pub mod text_box;

use slotmap::SlotMap;

pub use image_box::*;
pub use text_box::*;

use crate::{measures::Dimensions, PageId};

/// Centimeters per inch, used to express physical page sizes (traditionally
/// specified in inches) in the document's world units (1 unit = 1 cm).
pub const CM_PER_INCH: f32 = 2.54;

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

    /// Are page-level resources dirty? (e.g. we need to recreate the vbuf)
    pub elem_dirty: bool,
    /// Are setting-level resources dirty? (e.g. we need to recreate the uniform)
    pub is_dirty: bool,
}

impl Default for PrintLayout {
    fn default() -> Self {
        let mut pages = SlotMap::with_key();
        pages.insert(Page { pos: cgmath::Point2 { x: 0.0, y: 0.0 }, label: None });
        // pages.insert(Page { pos: cgmath::Point2 { x: 1.05, y: 0.0 }, label: None });
        Self {
            page_size: Default::default(),
            page_margin_start: cgmath::Point2 { x: 0.5 * CM_PER_INCH, y: 0.5 * CM_PER_INCH },
            page_margin_end: cgmath::Point2 { x: 0.5 * CM_PER_INCH, y: 0.5 * CM_PER_INCH },
            pages,
            elem_dirty: true,
            is_dirty: true,
        }
    }
}
