use pp_core::MaterialId;
use pp_editor::state::SelectionMode;

use crate::{cache, gpu};

mod flaps;
mod flaps_lines;
mod lines;
mod lines_cut;
mod points;
mod surface;
mod tris;

/// What a draw *is*, which fixes where it sits in the stack of coplanar
/// geometry. Ordered back-to-front.
///
/// A class is turned into a depth offset by the shaders' shared
/// `_apply_depth_offset`, which lifts geometry toward the eye by one
/// `DEPTH_CLASS_STEP` per class — as a fraction of view depth, so it holds at
/// any zoom and stays small against real depth differences. This replaces
/// `wgpu::DepthBiasState`, whose constant term is a no-op at these magnitudes on
/// a `Depth32Float` target and whose slope-scaled term vanishes on the
/// camera-facing geometry of the piece view — exactly where it was needed.
///
/// The ordering is global, not per-object: every flap in the document sits
/// behind every surface, and every line in front of every surface, whichever
/// piece owns it. That's what keeps a tab beneath the faces of the piece it
/// overlaps — a tab is paper tucked under the next piece, never over it.
///
/// Within one class, pieces are separated by their own slot (see
/// `PieceUniform::depth_slot`), which only breaks ties between coplanar pieces
/// and can never push one out of its class.
#[repr(u32)]
#[derive(Debug, Clone, Copy, Default)]
pub enum DepthClass {
    /// Tab paper. Always beneath any face.
    FlapFill = 0,
    /// Tab borders. Above their own fill, still beneath any face.
    FlapOutline = 1,
    /// Textured faces — the reference plane everything else is placed against.
    #[default]
    Surface = 2,
    /// The translucent selection tint drawn over the surface.
    FaceOverlay = 3,
    /// Interior fold annotations.
    FoldLine = 4,
    /// Cut and boundary lines: the piece silhouette.
    CutLine = 5,
    /// Vertex points.
    Vertex = 6,
}

impl DepthClass {
    /// The pipeline-overridable constants placing a pipeline in this class.
    /// Pass to a `VertexState`'s `compilation_options`; the fragment stage
    /// doesn't read it, so it only needs to go on the vertex side.
    pub fn compilation_options(&self) -> wgpu::PipelineCompilationOptions<'static> {
        wgpu::PipelineCompilationOptions {
            constants: match self {
                Self::FlapFill => &[("depth_class", 0.0)],
                Self::FlapOutline => &[("depth_class", 1.0)],
                Self::Surface => &[("depth_class", 2.0)],
                Self::FaceOverlay => &[("depth_class", 3.0)],
                Self::FoldLine => &[("depth_class", 4.0)],
                Self::CutLine => &[("depth_class", 5.0)],
                Self::Vertex => &[("depth_class", 6.0)],
            },
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct InkEngine {
    // Mesh draw programs
    points: points::PointsProgram,
    lines: lines::LinesProgram,
    lines_cut: lines_cut::LinesCutProgram,
    tris: tris::TrisProgram,
    surface: surface::SurfaceProgram,
    flaps: flaps::FlapsProgram,
    flaps_lines: flaps_lines::FlapsLinesProgram,
}

impl InkEngine {
    /// Builds the mesh draw programs against a given MSAA sample count.
    ///
    /// The count is baked into every pipeline, so a pass targeting a texture
    /// with a different sample count needs its own engine. The viewport passes
    /// the user's `msaa_level`; the print pass renders at 1x, where a 300 DPI
    /// pixel is already far finer than anything antialiasing would rescue.
    pub fn new(ctx: &gpu::Context, sample_count: u32) -> Self {
        Self {
            lines: lines::LinesProgram::new(ctx, sample_count),
            lines_cut: lines_cut::LinesCutProgram::new(ctx, sample_count),
            points: points::PointsProgram::new(ctx, sample_count),
            tris: tris::TrisProgram::new(ctx, sample_count),
            surface: surface::SurfaceProgram::new(ctx, sample_count),
            flaps: flaps::FlapsProgram::new(ctx, sample_count),
            flaps_lines: flaps_lines::FlapsLinesProgram::new(ctx, sample_count),
        }
    }

    /// Draws only the parts of the mesh using the specified material
    pub fn draw_mesh_for_material(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        material_id: &MaterialId,
    ) {
        self.surface.draw_mesh_with_material(ctx, render_pass, mesh, material_id);
    }

    pub fn draw_piece_mesh_for_material(
        &self,
        ctx: &gpu::Context,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        material_id: &MaterialId,
    ) {
        self.surface.draw_piece_mesh_with_material(ctx, render_pass, mesh, material_id);
    }

    pub fn draw_mesh(
        &self,
        ctx: &gpu::Context,
        selection_mode: &SelectionMode,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
        xray: bool,
    ) {
        if xray {
            // occluded wireframe elements go over the surface in xray mode
            if *selection_mode == SelectionMode::Vert {
                self.points.draw_mesh_xrayed(ctx, render_pass, mesh);
            }
            self.lines_cut.draw_mesh_xrayed(ctx, render_pass, mesh);
            if *selection_mode != SelectionMode::Piece {
                self.lines.draw_mesh_xrayed(ctx, render_pass, mesh);
            }
            self.tris.draw_mesh_xrayed(ctx, render_pass, mesh);
        };

        // always draw non-occluded elements
        self.tris.draw_mesh(ctx, render_pass, mesh);
        if *selection_mode != SelectionMode::Piece {
            self.lines.draw_mesh(ctx, render_pass, mesh);
        }
        self.lines_cut.draw_mesh(ctx, render_pass, mesh);
        if *selection_mode == SelectionMode::Vert {
            self.points.draw_mesh(ctx, render_pass, mesh);
        }
    }

    pub fn draw_piece_mesh(
        &self,
        ctx: &gpu::Context,
        selection_mode: &SelectionMode,
        render_pass: &mut wgpu::RenderPass,
        mesh: &cache::MeshGPU,
    ) {
        self.tris.draw_piece_mesh(ctx, render_pass, mesh);
        // Fold annotations describe the printout, so they only belong in Piece
        // mode. While editing verts / edges / faces, the plain wireframe is what
        // you need to see and click on.
        if *selection_mode == SelectionMode::Piece {
            self.lines.draw_piece_mesh_folds(ctx, render_pass, mesh);
        } else {
            self.lines.draw_piece_mesh(ctx, render_pass, mesh);
        }
        self.flaps.draw_piece_mesh(ctx, render_pass, mesh);
        self.flaps_lines.draw_piece_mesh(ctx, render_pass, mesh);
        if *selection_mode == SelectionMode::Vert {
            self.points.draw_piece_mesh(ctx, render_pass, mesh);
        }
    }
}
