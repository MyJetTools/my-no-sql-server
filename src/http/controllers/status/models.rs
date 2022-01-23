
#[derive(Serialize, Deserialize, Debug)]
struct NodeModel {}

#[derive(Serialize, Deserialize, Debug)]
struct LocationModel {
    pub id: String,
    pub compress: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueuesModel {
    pub persistence: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReaderModel {
    id: u64,
    pub name: String,
    pub ip: String,
    pub tables: Vec<String>,
    #[serde(rename = "lastIncomingTime")]
    pub last_incoming_time: String,
    #[serde(rename = "connectedTime")]
    pub connected_time: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct StatusModel {
    pub location: LocationModel,
    #[serde(rename = "masterNode")]
    pub master_node: Option<String>,
    pub nodes: Vec<NodeModel>,
    pub queues: QueuesModel,
    pub readers: Vec<ReaderModel>,
}

async fn get_readers(app: &AppContext) -> Vec<ReaderModel> {
    let mut result = Vec::new();
    let now = DateTimeAsMicroseconds::now();

    for session in app.data_readers.get_all().await {
        let metrics = session.metrics.get_metrics().await;

        result.push(ReaderModel {
            connected_time: metrics.connected.to_rfc3339(),
            last_incoming_time: format!("{:?}", now.duration_since(metrics.last_incoming_moment)),
            id: metrics.session_id,
            ip: metrics.ip.clone(),
            name: if let Some(name) = metrics.name {
                name
            } else {
                "???".to_string()
            },
            tables: session.get_tables().await,
        });
    }

    result
}