FROM rust
COPY . . 
ENTRYPOINT ["./target/release/my-nosql-server"]