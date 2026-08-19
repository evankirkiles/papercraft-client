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

    /// Starts a framing move that brings the current selection into view.
    ///
    /// Which cameras move depends on whether there *is* a selection, because
    /// the gesture means two different things:
    ///
    /// - With something selected, it's "show me this over there": every
    ///   viewport moves *except* the hovered one. That one is left alone
    ///   deliberately - the user is already looking where they want there, and
    ///   it's the other views that need to catch up to what was just selected.
    /// - With nothing selected, there is no "this" to show elsewhere, so it
    ///   reads as "fit what I'm looking at" and only the hovered viewport
    ///   moves, framing the whole document in its own space.
    ///
    /// With no viewport hovered there is nothing to single out either way, so
    /// every camera frames the document.
    ///
    /// The two viewport kinds draw the document in different spaces - folded
    /// meshes vs. unfolded pieces - so each derives its own bounds.
    pub fn frame_selection(&mut self, state: &pp_core::State) {
        // Outside vertex / edge mode, focus on faces alone: selecting verts
        // propagates up into the edge and face sets, so a face- or piece-mode
        // frame would otherwise be dragged off by leftovers from an earlier
        // vertex- or edge-mode selection that the user can no longer even see.
        let faces_only = !matches!(
            self.state.selection_mode,
            state::SelectionMode::Vert | state::SelectionMode::Edge
        );
        // Computed once here rather than per viewport: walking the pieces is
        // the expensive half of this.
        let bounds = state.selection_bounds(faces_only);
        let normal = state.selection_normal(faces_only);
        let piece_bounds = state.selection_piece_bounds(faces_only);
        let fit_radius = state.world_bounds().bounding_radius();
        let active = self.active_viewport;
        // An empty selection flips which side of the hovered/not-hovered split
        // gets framed; see the doc comment above.
        let frame_active = state.selection_is_empty(faces_only);
        self.layout
            .viewports
            .iter_mut()
            .filter(|(v_id, _)| active.is_none_or(|active| (*v_id == active) == frame_active))
            .for_each(|(_, viewport)| {
                let aspect = viewport.aspect();
                match &mut viewport.content {
                    viewport::ViewportContent::Folding(folding) => {
                        folding.camera.animate_to_frame(&bounds, normal, aspect)
                    }
                    viewport::ViewportContent::Cutting(cutting) => {
                        cutting.camera.animate_to_frame(&piece_bounds, aspect, fit_radius)
                    }
                }
            });
    }

    /// Advances every viewport camera's in-flight framing move. Called once per
    /// rendered frame with the elapsed milliseconds since the last one.
    pub fn tick_cameras(&mut self, dt_ms: f32) {
        self.layout.viewports.values_mut().for_each(|viewport| viewport.tick_camera(dt_ms));
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{EuclideanSpace, InnerSpace, MetricSpace};
    use pp_core::{
        id::{FaceId, Id},
        select::SelectionActionType,
    };
    use viewport::camera::{animation::FRAME_DURATION_MS, perspective::PerspectiveCamera};

    use super::*;

    /// An editor sized like a real window, with the *cutting* viewport hovered
    /// - framing skips the hovered one, so this is what exercises the folding
    /// camera.
    fn editor() -> Editor {
        let mut editor = Editor::default();
        editor.resize(&Dimensions { width: 1600.0, height: 900.0 }, 1.0);
        editor.active_viewport = editor
            .layout
            .viewports
            .iter()
            .find(|(_, v)| matches!(v.content, viewport::ViewportContent::Cutting(_)))
            .map(|(id, _)| id);
        editor
    }

    fn folding_camera(editor: &Editor) -> viewport::camera::perspective::PerspectiveCamera {
        editor
            .layout
            .viewports
            .values()
            .find_map(|v| match &v.content {
                viewport::ViewportContent::Folding(folding) => Some(folding.camera),
                _ => None,
            })
            .expect("the default layout has a folding viewport")
    }

    fn cutting_camera(editor: &Editor) -> viewport::camera::orthographic::OrthographicCamera {
        editor
            .layout
            .viewports
            .values()
            .find_map(|v| match &v.content {
                viewport::ViewportContent::Cutting(cutting) => Some(cutting.camera),
                _ => None,
            })
            .expect("the default layout has a cutting viewport")
    }

    /// The whole path the `.` keybind runs: selection -> bounds -> destination
    /// -> animation -> per-frame ticks.
    #[test]
    fn framing_a_selected_face_looks_down_its_normal() {
        let mut state = pp_core::State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        // A side face of the cube, so its normal isn't the degenerate ±Z
        let f_id = FaceId::from_usize(4);
        let normal = cgmath::Vector3::from(state.meshes[m_id][f_id].no).normalize();
        assert!(normal.z.abs() < 0.5, "test premise: face {f_id:?} faces sideways");
        state.select_face(&(m_id, f_id), SelectionActionType::Select, false, true);

        let mut editor = editor();
        let before = folding_camera(&editor).eye;
        editor.frame_selection(&state);

        // Halfway through, the camera has moved but hasn't arrived
        editor.tick_cameras(FRAME_DURATION_MS / 2.0);
        let midway = folding_camera(&editor);
        assert!(midway.eye.distance(before) > 1e-3, "camera should have started moving");
        assert!(midway.animation.is_some(), "animation should still be running halfway through");

        editor.tick_cameras(FRAME_DURATION_MS / 2.0);
        let camera = folding_camera(&editor);
        assert!(camera.animation.is_none(), "animation should be done after its full duration");

        // The camera sits on the selection's normal, looking back down it
        let offset = (camera.eye - camera.target).normalize();
        assert!(
            (offset - normal).magnitude() < 1e-3,
            "camera approached from {offset:?}, expected the face normal {normal:?}"
        );
        let center = state.selection_bounds(false).center();
        assert!(camera.target.distance(cgmath::Point3::from_vec(center)) < 1e-4);
    }

    /// Outside vertex / edge mode, framing centers on the faces alone, so a
    /// vert left selected from an earlier mode can't pull the camera off.
    #[test]
    fn face_mode_framing_ignores_leftover_verts() {
        let mut state = pp_core::State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        let f_id = FaceId::from_usize(4);
        state.select_face(&(m_id, f_id), SelectionActionType::Select, false, true);
        let face_center = state.selection_bounds(true).center();

        // A stray vert on the opposite corner, as if selected before the user
        // switched to face mode
        let far = state.meshes[m_id]
            .verts
            .indices()
            .map(pp_core::id::VertexId::from_usize)
            .max_by(|a, b| {
                let key = |v| state.meshes[m_id].vert_pos(v).x;
                key(*a).partial_cmp(&key(*b)).unwrap()
            })
            .unwrap();
        state.select_vert(&(m_id, far), SelectionActionType::Select, false);

        let mut face_mode = editor();
        face_mode.state.selection_mode = state::SelectionMode::Face;
        face_mode.frame_selection(&state);
        face_mode.tick_cameras(FRAME_DURATION_MS);
        let face_mode_target = folding_camera(&face_mode).target;
        assert!(
            face_mode_target.distance(cgmath::Point3::from_vec(face_center)) < 1e-4,
            "face mode should center on the face, got {face_mode_target:?}"
        );

        // Vertex mode still takes the stray vert into account
        let mut vert_mode = editor();
        vert_mode.state.selection_mode = state::SelectionMode::Vert;
        vert_mode.frame_selection(&state);
        vert_mode.tick_cameras(FRAME_DURATION_MS);
        assert!(
            folding_camera(&vert_mode).target.distance(face_mode_target) > 1e-3,
            "vertex mode should have centered somewhere else"
        );
    }

    #[test]
    fn framing_leaves_the_hovered_viewport_alone() {
        // Something has to be selected: with an empty selection the gesture
        // means the opposite, and frames the hovered viewport instead.
        let mut state = pp_core::State::with_cube();
        let m_id = state.meshes.keys().next().unwrap();
        state.select_face(&(m_id, FaceId::from_usize(4)), SelectionActionType::Select, false, true);

        let mut editor = editor(); // hovering the cutting viewport
        let before = cutting_camera(&editor);

        editor.frame_selection(&state);
        editor.tick_cameras(FRAME_DURATION_MS);

        let after = cutting_camera(&editor);
        assert!(after.animation.is_none(), "the hovered camera should never animate");
        assert_eq!(after.eye, before.eye);
        assert_eq!(after.zoom, before.zoom);
        // ...while the viewport the user isn't looking at caught up
        assert_ne!(folding_camera(&editor).eye, PerspectiveCamera::default().eye);
    }

    #[test]
    fn framing_without_a_hovered_viewport_frames_every_one() {
        // A lone triangle, so the document has a piece for the cutting
        // viewport to frame as well as geometry for the folding one
        let mut state = pp_core::State::default();
        let m_id = state.meshes.insert(pp_core::mesh::Mesh::new_tri());
        state.meshes[m_id].expand_piece(FaceId::from_usize(0)).unwrap();

        let mut editor = editor();
        editor.active_viewport = None;

        editor.frame_selection(&state);

        assert!(folding_camera(&editor).animation.is_some());
        assert!(cutting_camera(&editor).animation.is_some());
    }

    /// With nothing selected, `.` is a "fit what I'm looking at" gesture, so it
    /// frames the hovered viewport - the exact opposite of the selected case.
    #[test]
    fn framing_an_empty_selection_frames_the_hovered_viewport() {
        // A lone triangle, so both viewports have something to frame
        let mut state = pp_core::State::default();
        let m_id = state.meshes.insert(pp_core::mesh::Mesh::new_tri());
        state.meshes[m_id].expand_piece(FaceId::from_usize(0)).unwrap();
        assert!(state.selection.faces.is_empty());

        // Hovering the *folding* viewport, so the camera under test is the one
        // the other tests treat as the bystander.
        let mut editor = editor();
        editor.active_viewport = editor
            .layout
            .viewports
            .iter()
            .find(|(_, v)| matches!(v.content, viewport::ViewportContent::Folding(_)))
            .map(|(id, _)| id);

        editor.frame_selection(&state);
        editor.tick_cameras(1000.0);

        let camera = folding_camera(&editor);
        let bounds = state.world_bounds();
        assert!(camera.target.distance(cgmath::Point3::from_vec(bounds.center())) < 1e-4);
        // Far enough back to see the whole thing
        assert!(camera.eye.distance(camera.target) > bounds.bounding_radius());
        // ...and the viewport the user is *not* looking at stayed put
        assert!(cutting_camera(&editor).animation.is_none());
    }

    /// The empty-selection case still frames every camera when the pointer is
    /// outside the viewports, since there is no hovered one to single out.
    #[test]
    fn framing_an_empty_selection_without_a_hovered_viewport_frames_every_one() {
        let mut state = pp_core::State::default();
        let m_id = state.meshes.insert(pp_core::mesh::Mesh::new_tri());
        state.meshes[m_id].expand_piece(FaceId::from_usize(0)).unwrap();
        assert!(state.selection.faces.is_empty());

        let mut editor = editor();
        editor.active_viewport = None;

        editor.frame_selection(&state);

        assert!(folding_camera(&editor).animation.is_some());
        assert!(cutting_camera(&editor).animation.is_some());
    }
}
