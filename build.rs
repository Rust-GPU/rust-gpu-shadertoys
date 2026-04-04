use spirv_builder::{SpirvBuilder, SpirvMetadata};
use std::error::Error;

fn build_shader(path_to_crate: &str) -> Result<(), Box<dyn Error>> {
    let mut builder = SpirvBuilder::new(path_to_crate, "spirv-unknown-vulkan1.2")
        .spirv_metadata(SpirvMetadata::Full);
    builder.build_script.env_shader_spv_path = Some(true);

    let _result = builder.build()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    build_shader("shaders")?;
    Ok(())
}
