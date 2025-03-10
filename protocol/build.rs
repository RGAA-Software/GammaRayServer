fn main() {
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["tc_protocol/relay_message.proto"], &["tc_protocol"])
        .unwrap();
}