use std::io::Write;

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
        content: &[u8],
    ) -> Result<(), zip::result::ZipError> {
        self.zip_writer.start_file(table_name, self.options)?;

        let mut pos = 0;
        while pos < content.len() {
            let size = self.zip_writer.write(&content[pos..])?;

            pos += size;
        }

        Ok(())
    }

    pub fn get_payload(&mut self) -> Result<Vec<u8>, zip::result::ZipError> {
        let result = self.zip_writer.finish()?;
        Ok(result.buf)
    }
}
