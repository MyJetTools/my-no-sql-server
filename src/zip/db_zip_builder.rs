use std::io::Write;

use my_no_sql_server_core::db_snapshots::DbTableSnapshot;

use super::VecWriter;

pub struct DbZipBuilder {
    zip_writer: zip::ZipWriter<VecWriter>,
    options: zip::write::FileOptions,
}

impl DbZipBuilder {
    pub fn new() -> Self {
        let result = Self {
            zip_writer: zip::ZipWriter::new(VecWriter::new()),
            options: zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        };

        result
    }

    pub fn add_table(
        &mut self,
        table_name: &str,
        content: &DbTableSnapshot,
    ) -> Result<(), zip::result::ZipError> {
        for (partition_key, content) in &content.by_partition {
            use base64::Engine;
            let encoded_file_name =
                base64::engine::general_purpose::STANDARD.encode(partition_key.as_bytes());
            let file_name = format!("{}/{}", table_name, encoded_file_name);

            self.zip_writer.start_file(file_name, self.options)?;

            let json = content.db_rows_snapshot.as_json_array();

            let payload = json.build();

            let mut pos = 0;
            while pos < payload.len() {
                let size = self.zip_writer.write(&payload[pos..])?;

                pos += size;
            }
        }

        Ok(())
    }

    pub fn get_payload(&mut self) -> Result<Vec<u8>, zip::result::ZipError> {
        let result = self.zip_writer.finish()?;
        Ok(result.buf)
    }
}
