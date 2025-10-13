use pp_core::id::{EdgeId, Id};
use pp_core::State;
use pp_save::load::Loadable;
use pp_save::save::Saveable;

fn main() {
    println!("=== GLTF Cuts & Pieces Round-trip Test ===\n");

    // Create a simple state with a cube
    let mut state = State::with_cube();

    // Cut all edges to create pieces
    let edge_ids: Vec<_> = state
        .meshes
        .iter()
        .flat_map(|(m_id, mesh)| {
            mesh.edges.indices().map(move |e_id| (m_id, EdgeId::from_usize(e_id)))
        })
        .collect();

    println!("Created original state with cube mesh");
    println!("  Meshes: {}", state.meshes.len());

    let (m_id, orig_mesh) = state.meshes.iter().next().unwrap();
    println!("  Vertices: {}", orig_mesh.verts.num_elements());
    println!("  Faces: {}", orig_mesh.faces.num_elements());
    println!("  Edges: {}", orig_mesh.edges.num_elements());

    let orig_mesh = &state.meshes[m_id];
    let orig_cuts_count = orig_mesh.edges.iter().filter(|(_, e)| e.cut.is_some()).count();
    let orig_pieces_count = orig_mesh.pieces.num_elements();

    println!("\nBefore cutting:");
    println!("  Cut edges: {}", orig_cuts_count);
    println!("  Pieces: {}", orig_pieces_count);
    println!("{:?}", orig_mesh.pieces);

    // Cut all edges
    println!("\nCutting all {} edges...", edge_ids.len());
    edge_ids.iter().for_each(|id| {
        state.cut_edge(id, pp_core::cut::CutActionType::Cut, None);
    });

    let orig_mesh = &state.meshes[m_id];
    let orig_cuts_count = orig_mesh.edges.iter().filter(|(_, e)| e.cut.is_some()).count();
    let orig_pieces_count = orig_mesh.pieces.num_elements();

    println!("After cutting:");
    println!("  Cut edges: {}", orig_cuts_count);
    println!("  Pieces: {}", orig_pieces_count);
    println!("{:?}", orig_mesh.pieces);

    // Save to GLTF format
    println!("\n--- Saving to GLTF format ---");
    let save_file = match state.save() {
        Ok(file) => {
            println!("✓ Successfully saved to GLTF format");
            file
        }
        Err(e) => {
            eprintln!("✗ Failed to save: {:?}", e);
            return;
        }
    };

    // Serialize to JSON string (optional - just to show the output)
    match save_file.to_json_string_pretty() {
        Ok(json) => {
            println!("\nGenerated GLTF JSON ({} bytes):", json.len());
            // Print first few lines
            for (i, line) in json.lines().take(15).enumerate() {
                println!("  {}: {}", i + 1, line);
            }
            println!("  ... (truncated)");
        }
        Err(e) => {
            eprintln!("Failed to serialize to JSON: {:?}", e);
        }
    }

    // Load back from GLTF format
    println!("\n--- Loading from GLTF format ---");
    let loaded_state = match State::load(save_file) {
        Ok(state) => {
            println!("✓ Successfully loaded from GLTF format");
            state
        }
        Err(e) => {
            eprintln!("✗ Failed to load: {:?}", e);
            return;
        }
    };

    // Compare stats
    println!("\n--- Comparison ---");
    println!("Original state:");
    println!("  Meshes: {}", state.meshes.len());
    println!("  Materials: {}", state.materials.len());
    println!("  Textures: {}", state.textures.len());
    println!("  Samplers: {}", state.samplers.len());
    println!("  Images: {}", state.images.len());

    println!("\nLoaded state:");
    println!("  Meshes: {}", loaded_state.meshes.len());
    println!("  Materials: {}", loaded_state.materials.len());
    println!("  Textures: {}", loaded_state.textures.len());
    println!("  Samplers: {}", loaded_state.samplers.len());
    println!("  Images: {}", loaded_state.images.len());

    // Compare mesh details
    if let (Some((_, orig_mesh)), Some((_, loaded_mesh))) =
        (state.meshes.iter().next(), loaded_state.meshes.iter().next())
    {
        let loaded_cuts_count = loaded_mesh.edges.iter().filter(|(_, e)| e.cut.is_some()).count();
        let loaded_pieces_count = loaded_mesh.pieces.num_elements();

        println!("\nMesh comparison:");
        println!(
            "  Original - Vertices: {}, Faces: {}, Edges: {}",
            orig_mesh.verts.num_elements(),
            orig_mesh.faces.num_elements(),
            orig_mesh.edges.num_elements()
        );
        println!(
            "  Loaded   - Vertices: {}, Faces: {}, Edges: {}",
            loaded_mesh.verts.num_elements(),
            loaded_mesh.faces.num_elements(),
            loaded_mesh.edges.num_elements()
        );

        println!("\nCuts & Pieces comparison:");
        println!("  Original - Cut edges: {}, Pieces: {}", orig_cuts_count, orig_pieces_count);
        println!("  Loaded   - Cut edges: {}, Pieces: {}", loaded_cuts_count, loaded_pieces_count);
        println!("{:?}", orig_mesh.pieces);
        println!("{:?}", loaded_mesh.pieces);

        // Check if everything matches
        let topology_match = orig_mesh.verts.num_elements() == loaded_mesh.verts.num_elements()
            && orig_mesh.faces.num_elements() == loaded_mesh.faces.num_elements()
            && orig_mesh.edges.num_elements() == loaded_mesh.edges.num_elements();

        let cuts_match = orig_cuts_count == loaded_cuts_count;
        let pieces_match = orig_pieces_count == loaded_pieces_count;

        println!("\n--- Results ---");
        println!("  Topology preserved: {}", if topology_match { "✓" } else { "✗" });
        println!("  Cuts preserved: {}", if cuts_match { "✓" } else { "✗" });
        println!("  Pieces preserved: {}", if pieces_match { "✓" } else { "✗" });

        if topology_match && cuts_match && pieces_match {
            println!("\n✓ Round-trip successful! All data preserved.");
        } else {
            println!("\n⚠ Some data was not preserved during round-trip");
        }
    }

    println!("\n=== Test complete ===");
}
