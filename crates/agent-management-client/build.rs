fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .build_transport(true)
        .compile_protos(&["proto/agent_management.proto"], &["proto/"])?;
    Ok(())
}
