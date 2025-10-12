use pp_core::State;
use pp_io::{gltf::ImportGLTF, ppr::PprExport};

fn main() {
    // Create a simple state with a cube
    let mut state = State::default();
    state.import_gltf().expect("Failed to import GLTF");

    // Export to PPR format
    match state.export_ppr() {
        Ok(ppr_doc) => match ppr_doc.to_json() {
            Ok(json_string) => println!("{}", json_string),
            Err(e) => eprintln!("Failed to serialize to JSON: {}", e),
        },
        Err(_) => eprintln!("Failed to export PPR document"),
    }
}
