use std::{
    fs,
    io::{self, ErrorKind},
    path::{Component, Path, PathBuf},
};

use crate::file::{ARCHIVE_HEAD_SIZE, FileInfo, METADATA_SIZE, Metadata};

fn read_bytes(archive: &[u8], offset: u64, size: u64) -> io::Result<&[u8]> {
    let offset = usize::try_from(offset)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "archive offset is too large"))?;
    let size = usize::try_from(size)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "archive size is too large"))?;
    let end = offset
        .checked_add(size)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "archive range overflow"))?;

    archive
        .get(offset..end)
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "archive ended early"))
}

fn read_u32(archive: &[u8], offset: usize) -> io::Result<u32> {
    let bytes: [u8; 4] = archive
        .get(offset..offset + 4)
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "archive ended early"))?
        .try_into()
        .unwrap();

    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(archive: &[u8], offset: usize) -> io::Result<u64> {
    let bytes: [u8; 8] = archive
        .get(offset..offset + 8)
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "archive ended early"))?
        .try_into()
        .unwrap();

    Ok(u64::from_le_bytes(bytes))
}

fn read_metadata(archive: &[u8], offset: usize) -> io::Result<Metadata> {
    Ok(Metadata {
        files_number: read_u64(archive, offset)?,
        path_offset: read_u64(archive, offset + 8)?,
        path_size: read_u32(archive, offset + 16)?,
        content_offset: read_u64(archive, offset + 20)?,
        content_size: read_u64(archive, offset + 28)?,
        name_offset: read_u64(archive, offset + 36)?,
        name_size: read_u32(archive, offset + 44)?,
    })
}

fn validate_relative_path(path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "archive path is empty",
        ));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "archive path escapes the output directory",
                ));
            }
        }
    }

    Ok(())
}

pub fn read_archive(archive: &[u8]) -> io::Result<Vec<FileInfo>> {
    let metadata_size = usize::try_from(read_u32(archive, 0)?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "metadata size is too large"))?;
    let metadata_start = usize::try_from(read_u64(archive, 4)?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "metadata start is too large"))?;
    let metadata_item_size = METADATA_SIZE as usize;

    if metadata_start != ARCHIVE_HEAD_SIZE as usize || metadata_size % metadata_item_size != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "metadata size does not match archive format",
        ));
    }

    let files_number = metadata_size / metadata_item_size;
    let mut metadata = Vec::with_capacity(files_number);

    for index in 0..files_number {
        let offset = metadata_start
            .checked_add(index * metadata_item_size)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "metadata offset overflow"))?;

        let item = read_metadata(archive, offset)?;

        if item.files_number != files_number as u64 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "metadata file count does not match archive header",
            ));
        }

        metadata.push(item);
    }

    let mut files = Vec::with_capacity(files_number);

    for item in metadata {
        let name = String::from_utf8(
            read_bytes(archive, item.name_offset, item.name_size as u64)?.to_vec(),
        )
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "file name is not valid UTF-8"))?;

        let path = String::from_utf8(
            read_bytes(archive, item.path_offset, item.path_size as u64)?.to_vec(),
        )
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "file path is not valid UTF-8"))?;

        let path = PathBuf::from(path);
        validate_relative_path(&path)?;

        let content = read_bytes(archive, item.content_offset, item.content_size)?.to_vec();

        files.push(FileInfo {
            name,
            path,
            content,
        });
    }

    Ok(files)
}

pub fn restore_archive(archive: &[u8], output_root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let output_root = output_root.as_ref();
    let mut restored = Vec::new();

    for file in read_archive(archive)? {
        let output_path = output_root.join(&file.path);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output_path, &file.content)?;
        restored.push(output_path);
    }

    Ok(restored)
}
