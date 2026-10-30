use std::fs;
use std::path::Path;

use spirv_builder::{Capability, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let compile_result = SpirvBuilder::new(crate_dir, "spirv-unknown-vulkan1.2")
        .capability(Capability::RuntimeDescriptorArray)
        .capability(Capability::ShaderNonUniform)
        .capability(Capability::StorageBufferArrayNonUniformIndexing)
        .capability(Capability::SampledImageArrayNonUniformIndexing)
        .capability(Capability::StorageImageArrayNonUniformIndexing)
        .capability(Capability::StorageImageReadWithoutFormat)
        .capability(Capability::StorageImageWriteWithoutFormat)
        .capability(Capability::VulkanMemoryModel)
        .capability(Capability::VulkanMemoryModelDeviceScope)
        .capability(Capability::Int64)
        .capability(Capability::DrawParameters)
        .build()?;

    let entry_points = &compile_result.entry_points;
    for entry in ["mesh::mesh_vs", "mesh::mesh_fs", "cull::cull_cs"] {
        assert!(entry_points.contains(&entry.to_string()), "missing {entry} entry point, got {entry_points:?}");
    }

    let spv_path = compile_result.module.unwrap_single();
    let out_dir = crate_dir.join("out");
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("shaders.spv");
    fs::copy(spv_path, &out_path)?;

    println!("compiled {} entry points, wrote {}", entry_points.len(), out_path.display());
    Ok(())
}
