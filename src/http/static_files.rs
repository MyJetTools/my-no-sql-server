use tokio::fs::File;
use tokio::io::AsyncReadExt;

use super::{http_fail::HttpFailResult, http_ok::HttpOkResult};

pub async fn serve_path(path_and_file: &str) -> Result<HttpOkResult, HttpFailResult> {
    match load_file(path_and_file).await {
        Ok(content) => {
            let ok_result = HttpOkResult::Content {
                content,
                content_type: None,
            };

            Ok(ok_result)
        }
        Err(err) => {
            let err = HttpFailResult::as_not_found(format!(
                "Error handing file: {:?}. Filename: {}.",
                err, path_and_file
            ));

            Err(err)
        }
    }
}

pub async fn load_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let filename = format!("./wwwroot{}", filename);

    let mut file = File::open(&filename).await?;

    let mut result: Vec<u8> = Vec::new();

    loop {
        let res = file.read_buf(&mut result).await?;

        if res == 0 {
            break;
        }
    }

    return Ok(result);
}
