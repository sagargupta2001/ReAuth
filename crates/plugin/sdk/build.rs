use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This tells tonic-build to use the protoc binary downloaded by the vendored crate.
    env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["../../../proto/plugin.v1.proto"], &["../../../proto"])?;
    Ok(())
}