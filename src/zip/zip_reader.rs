use std::io::Read;

pub struct ZipReader {
    zip: zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
}

impl ZipReader {
    pub fn new(zip_content: Vec<u8>) -> Self {
        let zip_cursor = std::io::Cursor::new(zip_content);
        let zip = zip::ZipArchive::new(zip_cursor).unwrap();
        Self { zip }
    }

    pub fn get_file_names(&mut self) -> impl Iterator<Item = &str> {
        self.zip.file_names()
    }

    pub fn get_content_as_vec(&mut self, file_name: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut file = self.zip.by_name(file_name)?;
        let file_size = file.size() as usize;
        let mut content: Vec<u8> = Vec::with_capacity(file_size);

        let mut pos = 0;
        while pos < file_size {
            let size = file.read(&mut content[pos..])?;

            if size == 0 {
                break;
            }

            pos += size;
        }

        Ok(content)
    }
}
