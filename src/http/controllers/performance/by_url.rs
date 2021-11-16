use std::{collections::BTreeMap, vec};

use my_http_utils::HttpFailResult;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContext,
    http::{http_ok::HttpOkResult, metrics::HttpMetricsByUrl},
};

#[derive(Serialize, Deserialize, Debug)]
struct StatusCodeModel {
    #[serde(rename = "statusCode")]
    pub status_code: u8,
    pub durations: Vec<i64>,
}

impl StatusCodeModel {
    pub fn new(src: HttpMetricsByUrl) -> Vec<StatusCodeModel> {
        let mut as_tree_map: BTreeMap<u8, Vec<i64>> = BTreeMap::new();

        for item in src.items {
            if let Some(by_status_code) = as_tree_map.get_mut(&item.status_code) {
                by_status_code.push(item.microseconds);
            } else {
                let by_status_code = vec![item.microseconds];
                as_tree_map.insert(item.status_code, by_status_code);
            }
        }

        let mut result = Vec::new();

        for (status_code, durations) in as_tree_map {
            result.push(StatusCodeModel {
                status_code,
                durations,
            });
        }

        result
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GetByUrlModel {
    pub url: String,
    pub durations: Vec<StatusCodeModel>,
}

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let result = app.http_metrics.get_snapshot().await;

    let mut json_model = Vec::new();

    for (url, metrics) in result {
        json_model.push(GetByUrlModel {
            url,
            durations: StatusCodeModel::new(metrics),
        });
    }
    HttpOkResult::create_json_response(json_model)
}
