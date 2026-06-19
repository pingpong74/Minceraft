use std::path::Path;

fn compile_shader(slangc: &str, path: &Path, stem: &str, stage: &str, out_path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());

    let src_modified = path.metadata().and_then(|m| m.modified()).ok();
    let out_modified = out_path.metadata().and_then(|m| m.modified()).ok();

    let needs_compile = match (src_modified, out_modified) {
        (Some(src), Some(out)) => src > out,
        _ => !out_path.exists(),
    };

    if needs_compile {
        println!("cargo:warning=compiling {stem}.slang -> compiled/{stem}.spv");
        let status = std::process::Command::new(slangc)
            .arg(path)
            .arg("-o")
            .arg(out_path)
            .arg("-target")
            .arg("spirv")
            .arg("-entry")
            .arg("main")
            .arg("-stage")
            .arg(stage)
            .status()
            .expect("failed to run slangc; set SLANGC env var if not in PATH");

        if !status.success() {
            panic!("slangc failed for {stem}.slang");
        }
    }
}

fn main() {
    let shaders_dir = Path::new("shaders");
    let out_dir = shaders_dir.join("compiled");

    std::fs::create_dir_all(&out_dir).unwrap();

    let slangc = std::env::var("SLANGC").unwrap_or_else(|_| "slangc".to_string());

    let module_path = shaders_dir.join("sgpu.slang");
    println!("cargo:rerun-if-changed={}", module_path.display());

    for entry in std::fs::read_dir(shaders_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("slang") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_str().unwrap();
        if stem == "sgpu" || stem == "common" {
            continue;
        }

        let stage = match stem {
            "cull" => "compute",
            _ => panic!("unknown shader stage for root shader {stem}"),
        };

        let out_path = out_dir.join(format!("{stem}.spv"));
        compile_shader(&slangc, &path, stem, stage, &out_path);
    }

    let mesh_dir = shaders_dir.join("mesh");
    if mesh_dir.exists() {
        for entry in std::fs::read_dir(&mesh_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("slang") {
                continue;
            }

            let stem = path.file_stem().unwrap().to_str().unwrap();
            if stem == "sgpu" || stem == "common" {
                continue;
            }

            let stage = match stem {
                "vert" => "vertex",
                "frag" => "fragment",
                _ => panic!("unknown shader stage for mesh shader {stem}"),
            };

            let out_path = out_dir.join(format!("{stem}.spv"));
            compile_shader(&slangc, &path, stem, stage, &out_path);
        }
    }
}
