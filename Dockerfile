FROM rust:alpine
COPY . . 
ENTRYPOINT ["./target/release/my_no_sql_server"]