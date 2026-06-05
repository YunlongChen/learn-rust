fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .build_transport(true)
        .out_dir("src/generated")
        .compile_protos(&["proto/agent_management.proto"], &["proto/"])?;
    Ok(())
}
