use pp_core::State;
use pp_import::gltf::GlbSupport;
use pp_save::{load::Loadable, save::Saveable, SaveFile};

fn main() -> anyhow::Result<()> {
    // Import a simple GLB
    let file = SaveFile::from_glb(include_bytes!("./assets/Link.glb"))?;
    let mut state = State::load(file)?;

    // Export to PPR format
    match state.save() {
        Ok(ppr_doc) => match ppr_doc.to_json_string_pretty() {
            Ok(json_string) => println!("{}", json_string),
            Err(e) => eprintln!("Failed to serialize to JSON: {}", e),
        },
        Err(_) => eprintln!("Failed to export PPR document"),
    };

    Ok(())
}
