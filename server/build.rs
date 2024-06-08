fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = glob::glob("../protos/**/*.proto")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()?;
    let mut config = prost_build::Config::new();
    config.enable_type_names();
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/grpc")
        .include_file("mod.rs")
        .compile_with_config(config, files.as_slice(), &["../protos"])?;
    Ok(())
}
