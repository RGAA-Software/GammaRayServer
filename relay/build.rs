fn main() {
    prost_build::Config::new()
        .out_dir("src/proto")
        .compile_protos(&["tc_relay_proto/relay_message.proto"], &["tc_relay_proto"])
        .unwrap();
}