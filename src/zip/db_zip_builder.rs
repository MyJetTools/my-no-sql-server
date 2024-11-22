use std::io::Write;

use my_no_sql_server_core::db_snapshots::DbTableSnapshot;

use super::VecWriter;

pub struct DbZipBuilder {
    zip_writer: zip::ZipWriter<VecWriter>,
}

impl DbZipBuilder {
    pub fn new() -> Self {
        let result = Self {
            zip_writer: zip::ZipWriter::new(VecWriter::new()),
        };

        result
    }

    pub fn add_table(
        &mut self,
        table_name: &str,
        content: &DbTableSnapshot,
    ) -> Result<(), zip::result::ZipError> {
        let file_name = format!(
            "{}/{}",
            table_name,
            crate::scripts::TABLE_METADATA_FILE_NAME
        );

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        self.zip_writer.start_file(file_name, options)?;

        let payload = crate::scripts::serializers::table_attrs::serialize(&content.attr);
        write_to_zip_file(&mut self.zip_writer, &payload)?;

        for itm in &content.by_partition {
            use base64::Engine;
            let encoded_file_name = base64::engine::general_purpose::STANDARD
                .encode(itm.partition_key.as_str().as_bytes());
            let file_name = format!("{}/{}", table_name, encoded_file_name);

            self.zip_writer.start_file(file_name, options)?;

            let json = itm.db_rows_snapshot.as_json_array();

            let payload = json.build();

            write_to_zip_file(&mut self.zip_writer, &payload)?;
        }

        Ok(())
    }

    pub fn get_payload(self) -> Result<Vec<u8>, zip::result::ZipError> {
        let result = self.zip_writer.finish()?;
        Ok(result.buf)
    }
}

fn write_to_zip_file(
    zip_writer: &mut zip::ZipWriter<VecWriter>,
    payload: &[u8],
) -> Result<(), zip::result::ZipError> {
    let mut pos = 0;
    while pos < payload.len() {
        let size = zip_writer.write(&payload[pos..])?;

        pos += size;
    }

    Ok(())
}
