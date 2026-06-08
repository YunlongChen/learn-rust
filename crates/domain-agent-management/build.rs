use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the directory containing this build script (the crate root)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Find protoc in the manifest directory
    let protoc_path = manifest_dir.join("protoc.exe");
    if !protoc_path.exists() {
        return Err(format!(
            "protoc.exe not found at {}",
            protoc_path.display()
        ).into());
    }

    // Set PROTOC environment variable before calling compile_protos
    std::env::set_var("PROTOC", protoc_path.to_string_lossy().as_ref());

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .out_dir("src/generated")
        .compile_protos(&["proto/agent_management.proto"], &["proto/"])?;

    // Tell cargo to rerun this build script if proto files change
    println!("cargo:rerun-if-changed=proto/agent_management.proto");
    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
