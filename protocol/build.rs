fn main() {
    tonic_build::configure()
        .out_dir("src/")
        .compile_protos(&[
            "tc_protocol/grpc_base.proto",
            "tc_protocol/grpc_relay.proto", 
            "tc_protocol/grpc_profile.proto",
        ], &["tc_protocol"])
        .expect("Failed to compile proto files");
    
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["tc_protocol/relay_message.proto",
            "tc_protocol/spvr_inner.proto",
            "tc_protocol/sd_inner.proto",
        ], &["tc_protocol"])
        .unwrap();
}