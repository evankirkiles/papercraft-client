use cgmath::Point2;
use measures::Dimensions;
use slotmap::{new_key_type, SlotMap};
use viewport::{Viewport, ViewportBounds};
use windowing::{Split, ViewTreeNode};

pub mod measures;
pub mod scene;
pub mod tool;
pub mod viewport;
pub mod windowing;

new_key_type! {
    pub struct ViewportId;
    pub struct SplitId;
}

/// Represents the entire state of the "core" editor, the client-side view and
/// organization of any number of viewports.
#[derive(Debug)]
pub struct Editor {
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
            root_node: ViewTreeNode::Split(splits.insert(Split {
                ratio: 0.5,
                is_dirty: true,
                direction: windowing::SplitDirection::Horizontal,
                first: ViewTreeNode::Viewport(viewports.insert(Viewport {
                    bounds: ViewportBounds { area: dimensions.into(), dpr },
                    content: viewport::ViewportContent::Folding(Default::default()),
                })),
                second: ViewTreeNode::Viewport(viewports.insert(Viewport {
                    bounds: ViewportBounds { area: dimensions.into(), dpr },
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
    fn update(&mut self) {
        let nodes: Vec<_> = self.iter_nodes().collect();
        nodes.iter().for_each(|(area, node)| {
            if let windowing::ViewTreeNode::Viewport(v_id) = node {
                let viewport = self.viewports.get_mut(*v_id).unwrap();
                viewport.bounds.area = *area;
                viewport.bounds.dpr = self.dpr;
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
