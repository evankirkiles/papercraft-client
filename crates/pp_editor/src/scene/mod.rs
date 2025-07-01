use cutting::CuttingScene;
use folding::FoldingScene;

pub mod cutting;
pub mod folding;

#[derive(Debug, Clone)]
pub struct EditorScenes {
    /// Piece layout / assembly
    pub cutting: CuttingScene,
    /// Model preview / edge cutting
    pub folding: FoldingScene,
}
