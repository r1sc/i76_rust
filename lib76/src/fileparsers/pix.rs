use super::binary_reader::Readable;

pub struct FileInfo {
    pub filename: String,
    pub offset: u32,
    pub length: u32,
}

pub struct PIX {
    pub files: Vec<FileInfo>,
}

impl Readable for PIX {
    fn consume(reader: &mut super::binary_reader::BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut files = Vec::new();
        let _num_files = reader.read_line()?;

        while let Ok(line) = reader.read_line() {
            if line.len() == 0 {
                break;
            }
            
            let mut splitted = line.split_ascii_whitespace();
            let filename = splitted.next().unwrap().to_lowercase();
            let offset: u32 = splitted.next().unwrap().parse().unwrap();
            let length: u32 = splitted.next().unwrap().parse().unwrap();

            files.push(FileInfo {
                filename,
                offset,
                length,
            });
        }

        Ok(Self { files })
    }
}
