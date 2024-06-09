fn generate_auth_server() -> Result<(), Box<dyn std::error::Error>> {
    let auth_files = &["../protos/zk_auth.proto"];
    let mut config = prost_build::Config::new();
    config.enable_type_names();
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/grpc")
        .compile_with_config(config, auth_files, &["../protos"])?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_auth_server()
}
