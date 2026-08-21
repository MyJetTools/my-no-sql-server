use std::io::{Seek, Write};

use my_no_sql_sdk::server::db_snapshots::DbTableSnapshot;

use super::VecWriter;

/// Writes a snapshot of a database as a zip archive into any sink that can be
/// written and seeked — a file for the backup on disk, a `Vec<u8>` for the
/// download endpoint that has to answer with a body.
pub struct DbZipBuilder<TWriter: Write + Seek> {
    zip_writer: zip::ZipWriter<TWriter>,
}

impl DbZipBuilder<VecWriter> {
    pub fn in_memory() -> Self {
        Self::new(VecWriter::new())
    }

    pub fn get_payload(self) -> Result<Vec<u8>, zip::result::ZipError> {
        let result = self.zip_writer.finish()?;
        Ok(result.buf)
    }
}

impl<TWriter: Write + Seek> DbZipBuilder<TWriter> {
    pub fn new(writer: TWriter) -> Self {
        Self {
            zip_writer: zip::ZipWriter::new(writer),
        }
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

            write_to_zip_file(&mut self.zip_writer, payload.as_bytes())?;
        }

        Ok(())
    }

    /// Closes the archive and gives the sink back — for a file that is what the
    /// caller has to flush and sync before renaming it into place.
    pub fn finish(self) -> Result<TWriter, zip::result::ZipError> {
        self.zip_writer.finish()
    }
}

fn write_to_zip_file<TWriter: Write + Seek>(
    zip_writer: &mut zip::ZipWriter<TWriter>,
    payload: &[u8],
) -> Result<(), zip::result::ZipError> {
    let mut pos = 0;
    while pos < payload.len() {
        let size = zip_writer.write(&payload[pos..])?;

        pos += size;
    }

    Ok(())
}
