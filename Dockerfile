FROM rust
COPY . . 
ENTRYPOINT ["./target/release/my-no-sql-server"]