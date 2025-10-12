use pp_core::State;
use pp_save::load::Loadable;
use pp_save::save::Saveable;

fn main() {
    println!("=== GLTF Save/Load Round-trip Example ===\n");

    // Create a simple state with a cube
    let original_state = State::with_cube();
    println!("Created original state with cube mesh");
    println!("  Meshes: {}", original_state.meshes.len());
    println!(
        "  Vertices: {}",
        original_state.meshes.iter().next().map(|(_, m)| m.verts.num_elements()).unwrap_or(0)
    );
    println!(
        "  Faces: {}",
        original_state.meshes.iter().next().map(|(_, m)| m.faces.num_elements()).unwrap_or(0)
    );

    // Save to GLTF format
    println!("\n--- Saving to GLTF format ---");
    let save_file = match original_state.save() {
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
    println!("  Meshes: {}", original_state.meshes.len());
    println!("  Materials: {}", original_state.materials.len());
    println!("  Textures: {}", original_state.textures.len());
    println!("  Samplers: {}", original_state.samplers.len());
    println!("  Images: {}", original_state.images.len());

    println!("\nLoaded state:");
    println!("  Meshes: {}", loaded_state.meshes.len());
    println!("  Materials: {}", loaded_state.materials.len());
    println!("  Textures: {}", loaded_state.textures.len());
    println!("  Samplers: {}", loaded_state.samplers.len());
    println!("  Images: {}", loaded_state.images.len());

    // Compare mesh details
    if let (Some((_, orig_mesh)), Some((_, loaded_mesh))) =
        (original_state.meshes.iter().next(), loaded_state.meshes.iter().next())
    {
        println!("\nMesh comparison:");
        println!(
            "  Original - Vertices: {}, Faces: {}",
            orig_mesh.verts.num_elements(),
            orig_mesh.faces.num_elements()
        );
        println!(
            "  Loaded   - Vertices: {}, Faces: {}",
            loaded_mesh.verts.num_elements(),
            loaded_mesh.faces.num_elements()
        );

        // Check if vertex counts match
        if orig_mesh.verts.num_elements() == loaded_mesh.verts.num_elements()
            && orig_mesh.faces.num_elements() == loaded_mesh.faces.num_elements()
        {
            println!("\n✓ Round-trip successful! Mesh topology preserved.");
        } else {
            println!("\n⚠ Mesh topology differs after round-trip");
        }
    }

    println!("\n=== Example complete ===");
}
