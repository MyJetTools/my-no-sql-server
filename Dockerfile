FROM rust:slim
COPY ./target/release/my_no_sql_server ./target/release/my_no_sql_server 
ENTRYPOINT ["./target/release/my_no_sql_server"]