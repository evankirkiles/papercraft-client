//! Writing the print layout out as a PDF.
//!
//! Each sheet becomes one page carrying a single full-page image: the cutting
//! viewport's own render of that sheet, cut lines and folds included.
//!
//! Rasterizing the lines rather than stroking them as PDF vector is a deliberate
//! choice. Vector strokes would print crisper, but they cost more than they
//! return:
//!
//! - **PDF has no depth test.** The viewport resolves overlap with one
//!   (`pp_draw`'s `DepthClass`), which is what tucks a tab under the face that
//!   covers it. Reproducing that with paint order alone needs the page image to
//!   be transparent off-face, i.e. a soft mask — a second full-page image, for
//!   lines the raster layer already contains.
//! - **The line taxonomy would live twice.** Cut / mountain / valley and their
//!   dash patterns are decided in `lines.wgsl`; a vector backend has to mirror
//!   that, with nothing to catch it drifting.
//!
//! So the page here is exactly what the screen shows, for free. Machine-readable
//! cut geometry is a separate concern, computed rather than drawn — see
//! `docs/cut-contours.md`.

use pdf_writer::{Filter, Finish, Name, Pdf, Ref, TextStr};
use pp_core::measures::Dimensions;

/// Centimeters per PDF point. PDF user space is 1/72 inch by definition.
const CM_PER_POINT: f32 = 2.54 / 72.0;

/// One sheet, ready to be written out.
pub struct PdfPage {
    /// The sheet's own size, in centimeters.
    pub size: Dimensions<f32>,
    /// The page's name in the layout, used as its PDF page label.
    pub label: Option<String>,
    /// The rendered sheet, covering the whole page.
    pub raster: RasterPage,
}

/// A rendered sheet, as opaque 8-bit RGB.
///
/// No alpha: the sheet is opaque paper, so the render is composited over white
/// before it gets here. That keeps the PDF free of soft masks, which print
/// drivers and RIPs handle with varying enthusiasm.
pub struct RasterPage {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

impl RasterPage {
    fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.rgb.len() == (self.width as usize * self.height as usize * 3)
    }
}

#[derive(Debug)]
pub enum PdfError {
    /// A page's pixel buffer doesn't match its stated dimensions.
    MalformedRaster { page: usize },
    /// There were no pages to write.
    NoPages,
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRaster { page } => {
                write!(f, "The rendered image for page {} was the wrong size", page + 1)
            }
            Self::NoPages => write!(f, "There were no pages to print"),
        }
    }
}

/// Object ids, laid out so every page's are a fixed stride apart.
const CATALOG_ID: Ref = Ref::new(1);
const PAGE_TREE_ID: Ref = Ref::new(2);
const PAGE_LABELS_ID: Ref = Ref::new(3);
/// Ids from here on are per page: the page, its content, and its image.
const FIRST_PAGE_ID: i32 = 4;
const REFS_PER_PAGE: i32 = 3;

/// Writes `pages` out as a single PDF.
pub fn write_pdf(pages: &[PdfPage]) -> Result<Vec<u8>, PdfError> {
    if pages.is_empty() {
        return Err(PdfError::NoPages);
    }
    for (i, page) in pages.iter().enumerate() {
        if !page.raster.is_valid() {
            return Err(PdfError::MalformedRaster { page: i });
        }
    }

    let mut pdf = Pdf::new();
    let page_ids: Vec<Ref> =
        (0..pages.len()).map(|i| Ref::new(FIRST_PAGE_ID + i as i32 * REFS_PER_PAGE)).collect();

    {
        let mut catalog = pdf.catalog(CATALOG_ID);
        catalog.pages(PAGE_TREE_ID);
        if pages.iter().any(|page| page.label.is_some()) {
            catalog.pair(Name(b"PageLabels"), PAGE_LABELS_ID);
        }
    }

    pdf.pages(PAGE_TREE_ID).kids(page_ids.iter().copied()).count(pages.len() as i32);
    write_page_labels(&mut pdf, pages);

    for (i, page) in pages.iter().enumerate() {
        let page_id = page_ids[i];
        let content_id = Ref::new(page_id.get() + 1);
        let image_id = Ref::new(page_id.get() + 2);

        let width_pt = page.size.width / CM_PER_POINT;
        let height_pt = page.size.height / CM_PER_POINT;

        {
            let mut writer = pdf.page(page_id);
            writer
                .parent(PAGE_TREE_ID)
                .media_box(pdf_writer::Rect::new(0.0, 0.0, width_pt, height_pt))
                .contents(content_id);
            writer.resources().x_objects().pair(Name(b"Im0"), image_id);
            writer.finish();
        }

        // The image fills the sheet, placed in untransformed PDF space so it
        // lands the right way up.
        let mut content = pdf_writer::Content::new();
        content.save_state();
        content.transform([width_pt, 0.0, 0.0, height_pt, 0.0, 0.0]);
        content.x_object(Name(b"Im0"));
        content.restore_state();
        pdf.stream(content_id, &content.finish());

        // FlateDecode rather than DCTDecode: the page is flat-shaded regions,
        // hard texture edges and thin lines, all of which JPEG rings around, and
        // lossless costs little on the kind of image a papercraft page is.
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&page.raster.rgb, 7);
        let mut image = pdf.image_xobject(image_id, &compressed);
        image
            .width(page.raster.width as i32)
            .height(page.raster.height as i32)
            .color_space()
            .device_rgb();
        image.bits_per_component(8).filter(Filter::FlateDecode);
        image.finish();
    }

    Ok(pdf.finish())
}

/// Names each page after the sheet it came from, when the layout named it.
///
/// These are the labels the old archive spent on filenames. A viewer shows them
/// in place of a bare page number, so "the flap on Body-Top" stays findable once
/// the layout has more than a handful of sheets.
///
/// `/PageLabels` is a number tree keyed by page index, and a `/P` prefix with no
/// numbering style gives the page exactly that name. Skipped entirely when no
/// sheet is named, since an empty tree is worse than none.
fn write_page_labels(pdf: &mut Pdf, pages: &[PdfPage]) {
    if pages.iter().all(|page| page.label.is_none()) {
        return;
    }
    let mut labels = pdf.indirect(PAGE_LABELS_ID).dict();
    let mut nums = labels.insert(Name(b"Nums")).array();
    for (i, page) in pages.iter().enumerate() {
        if let Some(label) = &page.label {
            nums.item(i as i32);
            nums.push().dict().pair(Name(b"P"), TextStr(label));
        }
    }
    nums.finish();
    labels.finish();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_page(width: f32, height: f32) -> PdfPage {
        PdfPage {
            size: Dimensions { width, height },
            label: None,
            raster: RasterPage { width: 2, height: 2, rgb: vec![255; 2 * 2 * 3] },
        }
    }

    /// A4 is 21 x 29.7cm, which is 595.3 x 841.9 PDF points. Getting this wrong
    /// scales the whole sheet, so pieces meeting at a fold would no longer meet.
    #[test]
    fn a4_comes_out_at_the_right_number_of_points() {
        let page = blank_page(21.0, 29.7);
        let width_pt = page.size.width / CM_PER_POINT;
        let height_pt = page.size.height / CM_PER_POINT;
        assert!((width_pt - 595.28).abs() < 0.1, "got {width_pt}");
        assert!((height_pt - 841.89).abs() < 0.1, "got {height_pt}");
        let bytes = write_pdf(&[page]).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("/MediaBox [0 0 595.27563 841.88983]"));
    }

    /// The file should be a PDF with one page per sheet.
    #[test]
    fn a_two_page_document_writes_two_pdf_pages() {
        let pages = vec![blank_page(21.0, 29.7), blank_page(21.0, 29.7)];
        let bytes = write_pdf(&pages).unwrap();
        assert!(bytes.starts_with(b"%PDF-"), "should be a PDF");
        assert!(String::from_utf8_lossy(&bytes).contains("/Count 2"));
    }

    /// The page image is compressed. Uncompressed, an A4 sheet at 300 DPI is
    /// 26MB, which is not a file anyone wants to download per print.
    #[test]
    fn the_page_image_is_compressed() {
        let mut page = blank_page(21.0, 29.7);
        let (w, h) = (256u32, 256u32);
        page.raster = RasterPage { width: w, height: h, rgb: vec![255; (w * h * 3) as usize] };
        let bytes = write_pdf(&[page]).unwrap();
        assert!(
            bytes.len() < (w * h * 3) as usize / 10,
            "expected the image to compress well, got {} bytes",
            bytes.len()
        );
        assert!(String::from_utf8_lossy(&bytes).contains("/Filter /FlateDecode"));
    }

    /// A named sheet keeps its name in the PDF, which is what the archive's
    /// filenames used to carry.
    #[test]
    fn a_named_sheet_becomes_a_pdf_page_label() {
        let mut page = blank_page(21.0, 29.7);
        page.label = Some("Body-Top".to_string());
        let bytes = write_pdf(&[page]).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/PageLabels"), "the catalog should point at the labels");
        assert!(text.contains("(Body-Top)"), "and the sheet's own name should be in them");
    }

    /// An unnamed layout writes no label tree at all: an empty one is worse than
    /// none, since a viewer would show blank names instead of page numbers.
    #[test]
    fn an_unnamed_layout_writes_no_label_tree() {
        let bytes = write_pdf(&[blank_page(21.0, 29.7)]).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("/PageLabels"));
    }

    /// A page whose pixel buffer doesn't match its dimensions is refused rather
    /// than written out as a corrupt image.
    #[test]
    fn a_malformed_raster_is_rejected() {
        let mut page = blank_page(21.0, 29.7);
        page.raster.rgb.truncate(5);
        assert!(matches!(write_pdf(&[page]), Err(PdfError::MalformedRaster { page: 0 })));
    }

    /// Nothing to print is an error, not an empty file a viewer would reject.
    #[test]
    fn an_empty_document_is_refused() {
        assert!(matches!(write_pdf(&[]), Err(PdfError::NoPages)));
    }
}
