FROM rust
COPY . . 
ENTRYPOINT ["./target/release/my_no_sql_server"]