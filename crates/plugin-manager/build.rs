use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["../../proto/plugin.v1.proto"], &["../../proto"])?;
    Ok(())
}