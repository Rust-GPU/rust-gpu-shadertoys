use spirv_builder::{MetadataPrintout, SpirvBuilder};
use std::error::Error;

fn build_shader(path_to_crate: &str) -> Result<(), Box<dyn Error>> {
    let builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full);

    #[cfg(not(target_os = "macos"))]
    let builder = builder.shader_crate_features(["broken-on-metal".to_string()]);

    let _result = builder.build()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    build_shader("shaders")?;
    Ok(())
}
