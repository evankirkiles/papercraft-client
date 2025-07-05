use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use tsify::Tsify;

use crate::{measures::Rect, Editor, SplitId, ViewportId};

/// An axis upon which the viewport can be split
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Tsify, Serialize, Deserialize)]
pub enum SplitDirection {
    #[default]
    Horizontal, // top/bottom
    Vertical, // left/right
}

#[derive(Debug, Clone, Tsify, Serialize, Deserialize)]
pub struct Split {
    /// From 0-1, the ratio of the split
    pub ratio: f32,
    /// Whether this split is horizontal or vertical
    pub direction: SplitDirection,
    /// The left / top node in the split
    pub first: ViewTreeNode,
    /// The right / bottom node in the split
    pub second: ViewTreeNode,
    /// Indicates that this split's data has changed
    pub is_dirty: bool,
}

/// An atom in the window layout engine, either a split of two viewports or
/// a viewport itself.
#[derive(Debug, Clone, Copy, Tsify, Serialize, Deserialize)]
pub enum ViewTreeNode {
    Viewport(ViewportId),
    Split(SplitId),
}

impl Editor {
    pub fn iter_nodes(&self) -> WindowingNodeWalker<'_> {
        WindowingNodeWalker {
            frontier: Vec::from([(self.dimensions.into(), &self.root_node)]),
            splits: &self.splits,
        }
    }
}

/// Iterates all the viewports in the window in English reading order (LTR, TTB),
/// providing their computed areas.
#[derive(Debug, Clone)]
pub struct WindowingNodeWalker<'window> {
    /// The nodes waiting to be explored, plus their dimensions
    frontier: Vec<(Rect<f32>, &'window ViewTreeNode)>,
    splits: &'window SlotMap<SplitId, Split>,
}

impl<'window> Iterator for WindowingNodeWalker<'window> {
    type Item = (Rect<f32>, ViewTreeNode);
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.frontier.pop()?;
        let (area, node) = node;
        if let ViewTreeNode::Split(split_id) = node {
            let split = self.splits.get(*split_id).unwrap();
            let (first, second) =
                area.split(split.ratio, split.direction == SplitDirection::Vertical);
            self.frontier.push((second, &split.second));
            self.frontier.push((first, &split.first));
        };
        Some((area, *node))
    }
}
