//! Exporting the print layout as a PDF the user can print.
//!
//! A print run renders one page per frame, driven from [`crate::App::update`]
//! rather than an `async fn`: `map_async` never completes synchronously, and
//! wasm-bindgen holds a borrow on the exported `App` for the whole duration of
//! an async method - which the `requestAnimationFrame` loop would immediately
//! trip over. Spreading the run across frames also keeps the UI responsive
//! while a large grid rasterizes.
//!
//! Each page is the cutting viewport's own render of that sheet - cut lines and
//! folds included - placed into the PDF as an image. See [`pp_save::pdf`] for why
//! the lines are rasterized rather than stroked as vector.

use std::collections::VecDeque;

use pp_core::measures::{Dimensions, Rect};
use pp_draw::print::{PrintPoll, PrintTarget};
use pp_save::pdf::{self, PdfPage, RasterPage};
use wasm_bindgen::JsValue;

use crate::editor::trigger_download;

/// The name of the document a print run downloads.
const DOCUMENT_NAME: &str = "pages.pdf";

/// A sheet waiting to be rasterized: where it is in the world, and what it is
/// called.
pub(crate) struct PendingPage {
    label: Option<String>,
    rect: Rect<f32>,
}

/// A print run in flight: the pages still to render, the ones finished so far,
/// and the promise handed to JS when it started.
pub(crate) struct PrintJob {
    target: PrintTarget,
    /// The sheet size in centimeters, which every page shares.
    page_size: Dimensions<f32>,
    /// Pages still to render, in reading order.
    pending: VecDeque<PendingPage>,
    /// The page currently on the GPU, if any.
    in_flight: Option<PendingPage>,
    /// The pages finished so far, in reading order.
    done: Vec<PdfPage>,
    resolve: js_sys::Function,
    reject: js_sys::Function,
}

impl std::fmt::Debug for PrintJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrintJob")
            .field("pending", &self.pending.len())
            .field("in_flight", &self.in_flight.as_ref().map(|page| &page.label))
            .field("done", &self.done.len())
            .finish()
    }
}

impl PrintJob {
    pub(crate) fn new(
        target: PrintTarget,
        page_size: Dimensions<f32>,
        pending: VecDeque<PendingPage>,
        resolve: js_sys::Function,
        reject: js_sys::Function,
    ) -> Self {
        Self { target, page_size, pending, in_flight: None, done: Vec::new(), resolve, reject }
    }

    /// Advances the run by one step, returning `false` once it is over (either
    /// way) and the job should be dropped.
    ///
    /// At most one page is in flight at a time, since they share the target's
    /// readback buffer.
    pub(crate) fn tick(&mut self, renderer: &mut pp_draw::Renderer<'static>) -> bool {
        match renderer.print_poll(&mut self.target) {
            PrintPoll::Pending => return true,
            PrintPoll::Ready(Err(err)) => {
                self.settle(&self.reject, &JsValue::from_str(&err.to_string()));
                return false;
            }
            PrintPoll::Ready(Ok(rgb)) => {
                // `in_flight` is set whenever a page was submitted, so this
                // only misses if we polled a target nobody rendered into.
                if let Some(page) = self.in_flight.take() {
                    let Dimensions { width, height } = self.target.pixel_size();
                    self.done.push(PdfPage {
                        size: self.page_size,
                        label: page.label,
                        raster: RasterPage { width, height, rgb },
                    });
                }
            }
            PrintPoll::Idle => {}
        }

        let Some(page) = self.pending.pop_front() else {
            return self.finish();
        };
        let rect = page.rect;
        self.in_flight = Some(page);
        renderer.print_page(&mut self.target, rect);
        true
    }

    /// Writes the pages out as one PDF, hands it to the browser, and settles the
    /// promise. Always ends the job.
    fn finish(&self) -> bool {
        let write = pdf::write_pdf(&self.done).map_err(|err| JsValue::from_str(&err.to_string()));
        match write.and_then(|bytes| trigger_download(&bytes, DOCUMENT_NAME, "application/pdf")) {
            Ok(()) => self.settle(&self.resolve, &JsValue::from_f64(self.done.len() as f64)),
            Err(err) => self.settle(&self.reject, &err),
        }
        false
    }

    /// Rejects a job that can never make progress, e.g. because the canvas went
    /// away underneath it.
    pub(crate) fn abort(&self, reason: &str) {
        self.settle(&self.reject, &JsValue::from_str(reason));
    }

    fn settle(&self, callback: &js_sys::Function, value: &JsValue) {
        if let Err(err) = callback.call1(&JsValue::NULL, value) {
            log::error!("Failed to settle the print promise: {err:?}");
        }
    }
}

/// The pages of `state`'s print layout, in reading order.
pub(crate) fn pages_to_render(state: &pp_core::State) -> VecDeque<PendingPage> {
    let layout = &state.printing;
    layout
        .pages_in_grid_order()
        .into_iter()
        .map(|(col, row, id)| {
            let page = &layout.pages[id];
            PendingPage {
                label: Some(
                    page.label.clone().unwrap_or_else(|| format!("Page {}-{}", row + 1, col + 1)),
                ),
                rect: page.world_rect(&layout.page_size),
            }
        })
        .collect()
}
