use cgmath::Point2;
use pp_core::measures::Dimensions;
use serde::Serialize;
use slotmap::{new_key_type, SlotMap};
use tsify::Tsify;
use viewport::{Viewport, ViewportBounds};
use windowing::{Split, ViewTreeNode};

pub mod layout;
pub mod preferences;
pub mod scene;
pub mod state;
pub mod tool;
pub mod viewport;
pub mod windowing;

use layout::Layout;
use preferences::Preferences;
use state::EditorState;

new_key_type! {
    #[derive(Tsify)]
    pub struct ViewportId;
    #[derive(Tsify)]
    pub struct SplitId;
}

/// Represents the entire state of the "core" editor, the client-side view and
/// organization of any number of viewports.
#[derive(Debug, Tsify, Serialize)]
pub struct Editor {
    /// Long-term user preferences that persist across sessions
    pub preferences: Preferences,
    /// The window's layout: viewport tree, splits, and screen dimensions
    pub layout: Layout,
    /// Transient interaction state which drives rendering but doesn't persist
    pub state: EditorState,

    /// The current tool, which takes all input handling from the screen
    pub active_tool: Option<tool::Tool>,
    /// The current viewport, where input events are sent
    pub active_viewport: Option<ViewportId>,

    /// Whether the editor's state has changed since the last snapshot was
    /// sent to JS, used to know when to fire the `on_editor_state_change`
    /// callback.
    pub is_dirty: bool,
}

impl Default for Editor {
    fn default() -> Self {
        let dimensions: Dimensions<f32> = Default::default();
        let dpr: f32 = 1.0;
        let mut viewports: SlotMap<ViewportId, Viewport> = SlotMap::with_key();
        let mut splits: SlotMap<SplitId, Split> = SlotMap::with_key();
        let root_node = ViewTreeNode::Split(splits.insert(Split {
            ratio: 0.5,
            is_dirty: true,
            direction: windowing::SplitDirection::Horizontal,
            first: ViewTreeNode::Viewport(viewports.insert(Viewport {
                bounds: ViewportBounds { area: dimensions.into(), dpr, is_dirty: true },
                content: viewport::ViewportContent::Folding(Default::default()),
            })),
            second: ViewTreeNode::Viewport(viewports.insert(Viewport {
                bounds: ViewportBounds { area: dimensions.into(), dpr, is_dirty: true },
                content: viewport::ViewportContent::Cutting(Default::default()),
            })),
        }));
        Self {
            active_tool: None,
            active_viewport: None,
            is_dirty: false,
            preferences: Default::default(),
            state: Default::default(),
            layout: Layout { dimensions, dpr, root_node, splits, viewports },
        }
    }
}

impl Editor {
    /// Resets the editor state
    pub fn reset(&mut self) {
        self.active_tool = None;
        self.state.select_tool = Default::default();
    }

    /// Resizes the editor state, re-computing the dimensions of all nested viewports
    /// based on the new size of the editor.
    pub fn resize(&mut self, dims: &Dimensions<f32>, dpr: f32) {
        self.layout.dimensions = *dims;
        self.layout.dpr = dpr;
        self.update();
    }

    /// Walks the viewport tree and updates the stored sizes of any viewports
    /// whose dimensions have changed, marking them as needing re-layout. It
    /// also garbage collects any unreferenced viewports.
    pub fn update(&mut self) {
        let nodes: Vec<_> = self.iter_nodes().collect();
        let dpr = self.layout.dpr;
        nodes.iter().for_each(|(area, node)| {
            if let windowing::ViewTreeNode::Viewport(v_id) = node {
                let viewport = self.layout.viewports.get_mut(*v_id).unwrap();
                if viewport.bounds.area != *area || viewport.bounds.dpr != dpr {
                    viewport.bounds.area = *area;
                    viewport.bounds.dpr = dpr;
                    viewport.bounds.is_dirty = true;
                }
            }
        })
    }

    /// Gets which viewport is at the given position. We could do a binary search,
    /// but that's added complexity when users will typically have max 3 viewports to check.
    pub fn viewport_at(&self, pos: Point2<f32>) -> Option<ViewportId> {
        self.layout
            .viewports
            .iter()
            .find(|(_, viewport)| viewport.bounds.area.contains(&pos))
            .map(|(id, _)| id)
    }

    /// Refreshes every folding viewport's camera with the document's current
    /// bounding-sphere radius, called once per frame by the renderer so the
    /// far plane and dolly-out limit track the model even without the user
    /// interacting with the camera first (e.g. right after loading a model).
    pub fn sync_camera_bounds(&mut self, fit_radius: f32) {
        self.layout.viewports.values_mut().for_each(|viewport| {
            if let viewport::ViewportContent::Folding(folding) = &mut viewport.content {
                folding.camera.sync_fit_radius(fit_radius);
            }
        });
    }
}
