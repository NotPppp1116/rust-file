use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[repr(C)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: Vec<u8>,
}

pub fn read_dir(root: &Path) -> io::Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    // If the root directory itself cannot be opened, return error.
    let entries = fs::read_dir(root)?;

    for entry in entries {
        // If one entry is bad, skip only that one.
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        if !path.is_file() {
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
pub struct Metadata {
    files_number: u64,
    path_offset: u64,
    path_size: u32,
    content_offset: u64,
    content_size: u64,
    name_offset: u64,
    name_size: u32,
}

#[repr(C)]
pub struct ArchiveHead {
    metadata_size: u32,
    metadata_start: u64,
}

pub fn populate_metadata(info: &Vec<FileInfo>) -> Vec<Metadata> {
    let mut metadata_vec = Vec::new();

    let archive_head_size = size_of::<ArchiveHead>() as u64;
    let metadata_size = size_of::<Metadata>() as u64;
    let metadata_start = archive_head_size;

    let mut name_offset = metadata_start + metadata_size * info.len() as u64;

    let mut path_offset = name_offset;

    for item in info {
        path_offset += item.name.len() as u64;
    }

    let mut content_offset = path_offset;

    for item in info {
        content_offset += item.path.as_os_str().len() as u64;
    }

    for item in info {
        let metadata = Metadata {
            files_number: info.len() as u64,

            name_offset,
            name_size: item.name.len() as u32,

            path_offset,
            path_size: item.path.as_os_str().len() as u32,

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
        metadata_size: (size_of::<Metadata>() * blob.len()) as u32,
        metadata_start: size_of::<ArchiveHead>() as u64,
    }
}

pub fn append_struct<T>(out: &mut Vec<u8>, value: &T) {
    let bytes = unsafe {
        std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>())
    };

    out.extend_from_slice(bytes);
}
