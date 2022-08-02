fn main() {
    tonic_build::compile_protos("proto/MyNoSqlServer.proto").unwrap();
    tonic_build::compile_protos("proto/MyNoSqlServerPersistence.proto").unwrap();
}
