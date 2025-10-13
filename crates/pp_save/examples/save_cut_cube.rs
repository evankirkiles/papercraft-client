use pp_core::{
    id::{EdgeId, Id},
    State,
};
use pp_save::save::Saveable;

fn main() {
    // Create a simple state with a cube
    let mut state = State::with_cube();
    let edge_ids: Vec<_> = state
        .meshes
        .iter()
        .flat_map(|(m_id, mesh)| {
            mesh.edges.indices().map(move |e_id| (m_id, EdgeId::from_usize(e_id)))
        })
        .collect();
    edge_ids.iter().for_each(|id| {
        state.cut_edge(id, pp_core::cut::CutActionType::Cut, None);
    });

    // Export to PPR format
    match state.save() {
        Ok(ppr_doc) => match ppr_doc.to_json_string_pretty() {
            Ok(json_string) => println!("{}", json_string),
            Err(e) => eprintln!("Failed to serialize to JSON: {}", e),
        },
        Err(_) => eprintln!("Failed to export PPR document"),
    }
}
