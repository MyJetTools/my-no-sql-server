fn main() {
    tonic_build::compile_protos("proto/MyNoSqlServer.proto").unwrap();
}
