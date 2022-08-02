use std::{sync::Arc, time::Duration};

use my_no_sql_core::db::{
    db_snapshots::{DbPartitionSnapshot, DbTableSnapshot},
    DbRow, DbTableAttributes,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tonic::transport::Channel;

use crate::mynosqlserverpersistence_grpc::{
    my_no_sql_server_persistnce_grpc_service_client::MyNoSqlServerPersistnceGrpcServiceClient, *,
};

pub struct PersistGrpcService {
    timeout: Duration,
    channel: Channel,
}

impl PersistGrpcService {
    pub async fn new(grpc_address: String) -> Self {
        let channel = Channel::from_shared(grpc_address)
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            timeout: Duration::from_secs(5),
            channel,
        }
    }
    async fn create_grpc_service(&self) -> MyNoSqlServerPersistnceGrpcServiceClient<Channel> {
        MyNoSqlServerPersistnceGrpcServiceClient::new(self.channel.clone())
    }

    pub async fn get_tables(&self) -> Vec<TableDescriptionGrpcModel> {
        let mut client = self.create_grpc_service().await;

        let request_feature = client.get_tables(tonic::Request::new(()));

        let result = tokio::time::timeout(self.timeout, request_feature)
            .await
            .unwrap()
            .unwrap();

        let mut response_stream = result.into_inner();

        super::read_with_timeout::read_from_stream_to_vec(&mut response_stream, self.timeout).await
    }
    pub async fn get_table_rows(&self, table_name: String) -> Vec<TableEntityGrpcModel> {
        let mut client = self.create_grpc_service().await;

        let request_feature = client.get_table(GetTableGrpcRequest { table_name });

        let result = tokio::time::timeout(self.timeout, request_feature)
            .await
            .unwrap()
            .unwrap();

        let mut response_stream = result.into_inner();

        super::read_with_timeout::read_from_stream_to_vec(&mut response_stream, self.timeout).await
    }

    pub async fn clean_table(&self, table_name: String, persist_moment: DateTimeAsMicroseconds) {
        let mut client = self.create_grpc_service().await;

        let request_feature = client.clean_table(CleanTableGrpcRequest {
            table_name,
            persist_moment: persist_moment.unix_microseconds,
        });

        tokio::time::timeout(self.timeout, request_feature)
            .await
            .unwrap()
            .unwrap();
    }

    pub async fn init_table(&self, table_name: String, snapshot: DbTableSnapshot) {
        todo!("Implement")
    }

    pub async fn delete_partition(
        &self,
        table_name: &str,
        partition_key: &str,
    ) -> Vec<TableEntityGrpcModel> {
        todo!("Implement")
    }

    pub async fn init_partition(&self, table_name: &str, snapshot: DbPartitionSnapshot) {
        todo!("Implement")
    }

    pub async fn update_rows(&self, table_name: &str, db_rows: &[Arc<DbRow>]) {
        todo!("Implement")
    }

    pub async fn save_table_attrs(&self, table_name: &str, table_attrs: DbTableAttributes) {
        todo!("Implement")
    }
}
