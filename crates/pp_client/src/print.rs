//! Exporting the print layout's pages as images the user can print.
//!
//! A print run renders one page per frame, driven from [`crate::App::update`]
//! rather than an `async fn`: `map_async` never completes synchronously, and
//! wasm-bindgen holds a borrow on the exported `App` for the whole duration of
//! an async method - which the `requestAnimationFrame` loop would immediately
//! trip over. Spreading the run across frames also keeps the UI responsive
//! while a large grid rasterizes.

use std::{collections::VecDeque, io::Write};

use pp_core::measures::Rect;
use pp_draw::print::{PrintPoll, PrintTarget};
use wasm_bindgen::JsValue;

use crate::editor::trigger_download;

/// The name of the archive a print run downloads.
const ARCHIVE_NAME: &str = "pages.zip";

/// A print run in flight: the pages still to render, the images produced so
/// far, and the promise handed to JS when it started.
pub(crate) struct PrintJob {
    target: PrintTarget,
    /// Pages still to render, in reading order, as `(filename, world rect)`.
    pending: VecDeque<(String, Rect<f32>)>,
    /// The filename of the page currently on the GPU, if any.
    in_flight: Option<String>,
    /// The PNGs produced so far, in reading order.
    done: Vec<(String, Vec<u8>)>,
    resolve: js_sys::Function,
    reject: js_sys::Function,
}

impl std::fmt::Debug for PrintJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrintJob")
            .field("pending", &self.pending.len())
            .field("in_flight", &self.in_flight)
            .field("done", &self.done.len())
            .finish()
    }
}

impl PrintJob {
    pub(crate) fn new(
        target: PrintTarget,
        pending: VecDeque<(String, Rect<f32>)>,
        resolve: js_sys::Function,
        reject: js_sys::Function,
    ) -> Self {
        Self { target, pending, in_flight: None, done: Vec::new(), resolve, reject }
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
            PrintPoll::Ready(Ok(png)) => {
                // `in_flight` is set whenever a page was submitted, so this
                // only misses if we polled a target nobody rendered into.
                if let Some(name) = self.in_flight.take() {
                    self.done.push((name, png));
                }
            }
            PrintPoll::Idle => {}
        }

        let Some((name, page)) = self.pending.pop_front() else {
            return self.finish();
        };
        self.in_flight = Some(name);
        renderer.print_page(&mut self.target, page);
        true
    }

    /// Zips the rendered pages up, hands the archive to the browser, and
    /// settles the promise. Always ends the job.
    fn finish(&self) -> bool {
        match self.archive().and_then(|zip| trigger_download(&zip, ARCHIVE_NAME)) {
            Ok(()) => self.settle(&self.resolve, &JsValue::from_f64(self.done.len() as f64)),
            Err(err) => self.settle(&self.reject, &err),
        }
        false
    }

    /// Bundles the pages into a single archive.
    ///
    /// Stored rather than deflated: PNG is already compressed, so a second pass
    /// would spend time to save nothing.
    fn archive(&self) -> Result<Vec<u8>, JsValue> {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let to_js = |err: &dyn std::fmt::Display| JsValue::from_str(&format!("{err}"));
        for (name, png) in &self.done {
            writer.start_file(name, options).map_err(|e| to_js(&e))?;
            writer.write_all(png).map_err(|e| to_js(&e))?;
        }
        Ok(writer.finish().map_err(|e| to_js(&e))?.into_inner())
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

/// The pages of `state`'s print layout, in reading order, paired with the
/// filename each should take in the archive.
pub(crate) fn pages_to_render(state: &pp_core::State) -> VecDeque<(String, Rect<f32>)> {
    let layout = &state.printing;
    layout
        .pages_in_grid_order()
        .into_iter()
        .map(|(col, row, id)| {
            let page = &layout.pages[id];
            let name =
                page.label.clone().unwrap_or_else(|| format!("page-{}-{}", row + 1, col + 1));
            (format!("{name}.png"), page.world_rect(&layout.page_size))
        })
        .collect()
}
