use pp_core::State;
use pp_save::save::Saveable;

fn main() {
    // Create a simple state with a cube
    let mut state = State::with_cube();
    // state.import_gltf().expect("Failed to import GLTF");

    // Export to PPR format
    match state.save() {
        Ok(ppr_doc) => match ppr_doc.to_json() {
            Ok(json_string) => println!("{}", json_string),
            Err(e) => eprintln!("Failed to serialize to JSON: {}", e),
        },
        Err(_) => eprintln!("Failed to export PPR document"),
    }
}
