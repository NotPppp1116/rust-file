use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const ARCHIVE_HEAD_SIZE: u64 = 12;
pub const METADATA_SIZE: u64 = 48;

#[repr(C)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: Vec<u8>,
}

pub fn read_dir(root: &Path) -> io::Result<Vec<FileInfo>> {
    read_dir_from(root, root)
}

fn read_dir_from(root: &Path, current: &Path) -> io::Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    // If the root directory itself cannot be opened, return error.
    let entries = fs::read_dir(current)?;

    for entry in entries {
        // If one entry is bad, skip only that one.
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        if path.is_dir() {
            let Ok(mut sub_files) = read_dir_from(root, &path) else {
                continue;
            };
            files.append(&mut sub_files);
            continue;
        }

        // If this specific file cannot be read, skip only this file.
        let Ok(content) = fs::read(&path) else {
            continue;
        };

        let name = entry.file_name().to_string_lossy().into_owned();

        let relative_path = match path.strip_prefix(root) {
            Ok(p) => p.to_path_buf(),
            Err(_) => path.clone(),
        };

        files.push(FileInfo {
            name,
            path: relative_path,
            content,
        });
    }

    Ok(files)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub files_number: u64,
    pub path_offset: u64,
    pub path_size: u32,
    pub content_offset: u64,
    pub content_size: u64,
    pub name_offset: u64,
    pub name_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ArchiveHead {
    pub metadata_size: u32,
    pub metadata_start: u64,
}

pub fn populate_metadata(info: &Vec<FileInfo>) -> Vec<Metadata> {
    let mut metadata_vec = Vec::new();

    let metadata_start = ARCHIVE_HEAD_SIZE;

    let mut name_offset = metadata_start + METADATA_SIZE * info.len() as u64;

    let mut path_offset = name_offset;

    for item in info {
        path_offset += item.name.len() as u64;
    }

    let mut content_offset = path_offset;

    for item in info {
        content_offset += path_archive_bytes(&item.path).len() as u64;
    }

    for item in info {
        let metadata = Metadata {
            files_number: info.len() as u64,

            name_offset,
            name_size: item.name.len() as u32,

            path_offset,
            path_size: path_archive_bytes(&item.path).len() as u32,

            content_offset,
            content_size: item.content.len() as u64,
        };

        name_offset += metadata.name_size as u64;
        path_offset += metadata.path_size as u64;
        content_offset += metadata.content_size;

        metadata_vec.push(metadata);
    }

    metadata_vec
}

pub fn create_archive(blob: &[Metadata]) -> ArchiveHead {
    ArchiveHead {
        metadata_size: (METADATA_SIZE * blob.len() as u64) as u32,
        metadata_start: ARCHIVE_HEAD_SIZE,
    }
}

pub fn path_archive_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().into_owned().into_bytes()
}

pub fn append_archive_head(out: &mut Vec<u8>, value: &ArchiveHead) {
    out.extend_from_slice(&value.metadata_size.to_le_bytes());
    out.extend_from_slice(&value.metadata_start.to_le_bytes());
}

pub fn append_metadata(out: &mut Vec<u8>, value: &Metadata) {
    out.extend_from_slice(&value.files_number.to_le_bytes());
    out.extend_from_slice(&value.path_offset.to_le_bytes());
    out.extend_from_slice(&value.path_size.to_le_bytes());
    out.extend_from_slice(&value.content_offset.to_le_bytes());
    out.extend_from_slice(&value.content_size.to_le_bytes());
    out.extend_from_slice(&value.name_offset.to_le_bytes());
    out.extend_from_slice(&value.name_size.to_le_bytes());
}
