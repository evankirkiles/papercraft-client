use cgmath::Point2;
use pp_core::measures::Dimensions;
use serde::Serialize;
use settings::Settings;
use slotmap::{new_key_type, SlotMap};
use tsify::Tsify;
use viewport::{Viewport, ViewportBounds};
use windowing::{Split, ViewTreeNode};

pub mod scene;
pub mod settings;
pub mod tool;
pub mod viewport;
pub mod windowing;

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
    /// The current user-specific configuration of the editor
    pub settings: Settings,

    /// The window's full recursive tree layout, e.g. splits and viewports
    pub root_node: ViewTreeNode,
    /// Cuts where a node is split into separate viewports
    pub splits: SlotMap<SplitId, Split>,
    /// The leaves of the editor node tree where content is actually rendered
    pub viewports: SlotMap<ViewportId, Viewport>,

    /// The current tool, which takes all input handling from the screen
    pub active_tool: Option<tool::Tool>,
    /// The current viewport, where input events are sent
    pub active_viewport: Option<ViewportId>,

    /// Is this editor in "x-ray" mode?
    pub is_xray: bool,
    /// Is this editor in "presentation" mode?
    pub is_presentation: bool,
    /// The current dimensions of this editor
    pub dimensions: Dimensions<f32>,
    /// The DPI of this editor
    pub dpr: f32,
}

impl Default for Editor {
    fn default() -> Self {
        let dimensions: Dimensions<f32> = Default::default();
        let dpr: f32 = 1.0;
        let mut viewports: SlotMap<ViewportId, Viewport> = SlotMap::with_key();
        let mut splits: SlotMap<SplitId, Split> = SlotMap::with_key();
        Self {
            dpr,
            dimensions,
            active_tool: None,
            active_viewport: None,
            is_xray: false,
            is_presentation: false,
            settings: Default::default(),
            root_node: ViewTreeNode::Split(splits.insert(Split {
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
            })),
            splits,
            viewports,
        }
    }
}

impl Editor {
    /// Resizes the editor state, re-computing the dimensions of all nested viewports
    /// based on the new size of the editor.
    pub fn resize(&mut self, dims: &Dimensions<f32>, dpr: f32) {
        self.dimensions = *dims;
        self.dpr = dpr;
        self.update();
    }

    /// Walks the viewport tree and updates the stored sizes of any viewports
    /// whose dimensions have changed, marking them as needing re-layout. It
    /// also garbage collects any unreferenced viewports.
    pub fn update(&mut self) {
        let nodes: Vec<_> = self.iter_nodes().collect();
        nodes.iter().for_each(|(area, node)| {
            if let windowing::ViewTreeNode::Viewport(v_id) = node {
                let viewport = self.viewports.get_mut(*v_id).unwrap();
                if viewport.bounds.area != *area || viewport.bounds.dpr != self.dpr {
                    viewport.bounds.area = *area;
                    viewport.bounds.dpr = self.dpr;
                    viewport.bounds.is_dirty = true;
                }
            }
        })
    }

    /// Gets which viewport is at the given position. We could do a binary search,
    /// but that's added complexity when users will typically have max 3 viewports to check.
    pub fn viewport_at(&self, pos: Point2<f32>) -> Option<ViewportId> {
        self.viewports
            .iter()
            .find(|(_, viewport)| viewport.bounds.area.contains(&pos))
            .map(|(id, _)| id)
    }
}
