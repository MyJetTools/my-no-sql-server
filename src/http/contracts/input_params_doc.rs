use my_http_server::middlewares::controllers::documentation::{
    data_types::{HttpDataType, HttpField},
    in_parameters::{HttpInputParameter, HttpParameterInputSource},
};

pub fn table_name() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("name", HttpDataType::as_string(), true, None),
        description: "Name of a table".to_string(),
        source: HttpParameterInputSource::Query,
    }
}

pub fn max_partitions_amount() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new(
            super::input_params::PARAM_MAX_PARTITION_AMOUNTS,
            HttpDataType::as_long(),
            false,
            None,
        ),
        description: "Maximum partitions amount table is keeping".to_string(),
        source: HttpParameterInputSource::Query,
    }
}

pub fn persist_table() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new(
            super::input_params::PARAM_PERSIST_TABLE,
            HttpDataType::as_bool(),
            false,
            None,
        ),
        description: format!(
            "Should we persist table. If it's null - default value is {}",
            super::input_params::PERISTS_TABLE_DEFAULT
        ),
        source: HttpParameterInputSource::Query,
    }
}

pub fn sync_period() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("syncPeriod", HttpDataType::as_string(), true, None),
        description: format!(
            "Synchronization period. Default: {}",
            super::input_params::DEFAULT_SYNC_PERIOD.as_str()
        ),
        source: HttpParameterInputSource::Query,
    }
}

pub fn api_key() -> HttpInputParameter {
    HttpInputParameter {
        field: HttpField::new("apiKey", HttpDataType::as_string(), true, None),
        description: "Api key".to_string(),
        source: HttpParameterInputSource::Header,
    }
}