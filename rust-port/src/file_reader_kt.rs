//! FileReaderKt.java resource installation, with a Rust ZIP/JAR stream adapter.
#![allow(dead_code)]
use std::{
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};
#[derive(Debug, Default)]
pub(crate) struct ExtractionReport {
    pub written: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}
pub(crate) enum ResourceSource<'a> {
    Directory(&'a Path),
    Jar(&'a Path),
}
/// The default Kotlin filter accepts every path, including directories.
/// Directory read errors are reported and traversal continues, as in Java.
pub(crate) fn extract_files_into(
    source: ResourceSource<'_>,
    folder: &Path,
    filter: impl Fn(&Path) -> bool,
) -> io::Result<ExtractionReport> {
    let mut report = ExtractionReport::default();
    if let Err(error) = fs::create_dir_all(folder) {
        report
            .errors
            .push((folder.to_path_buf(), error.to_string()));
    }
    match source {
        ResourceSource::Directory(root) => walk(root, root, folder, &filter, &mut report)?,
        ResourceSource::Jar(archive) => {
            let bytes = fs::read(archive)?;
            let entries = jar_entries(&bytes)?;
            // The source ignores the URI entry and chooses /<destination basename>.
            let name = folder
                .file_name()
                .ok_or_else(|| invalid("Destination has no basename"))?
                .to_string_lossy();
            let prefix = format!("{name}/");
            if !entries
                .iter()
                .any(|entry| entry.name == name || entry.name.starts_with(&prefix))
            {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("JAR resource root /{name}"),
                ));
            }
            for entry in entries {
                let relative = if entry.name == name {
                    ""
                } else if let Some(path) = entry.name.strip_prefix(&prefix) {
                    path
                } else {
                    continue;
                };
                let virtual_path = PathBuf::from(format!("/{}", entry.name));
                if !filter(&virtual_path) {
                    continue;
                }
                let result = (|| {
                    let relative = Path::new(relative);
                    // Classpath ZIP paths must remain under their mounted resource root.
                    if relative
                        .components()
                        .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
                    {
                        return Err(invalid("Invalid resource path"));
                    }
                    if entry.name.ends_with('/') || relative.as_os_str().is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::IsADirectory,
                            "Cannot read directory as resource",
                        ));
                    }
                    let data = entry.decode(&bytes)?;
                    write_resource(folder.join(relative), &data, &mut report)
                })();
                if let Err(error) = result {
                    report.errors.push((virtual_path, error.to_string()));
                }
            }
        }
    }
    Ok(report)
}
pub(crate) fn extract_all(
    source: ResourceSource<'_>,
    folder: &Path,
) -> io::Result<ExtractionReport> {
    extract_files_into(source, folder, |_| true)
}
fn walk(
    root: &Path,
    path: &Path,
    folder: &Path,
    filter: &impl Fn(&Path) -> bool,
    report: &mut ExtractionReport,
) -> io::Result<()> {
    if filter(path) {
        let result = fs::read(path).and_then(|bytes| {
            write_resource(
                folder.join(
                    path.strip_prefix(root)
                        .map_err(|e| invalid(&e.to_string()))?,
                ),
                &bytes,
                report,
            )
        });
        if let Err(error) = result {
            report.errors.push((path.to_path_buf(), error.to_string()));
        }
    }
    // Files.walk does not follow directory symlinks.
    if fs::symlink_metadata(path)?.is_dir() {
        for entry in fs::read_dir(path)? {
            walk(root, &entry?.path(), folder, filter, report)?;
        }
    }
    Ok(())
}
fn write_resource(path: PathBuf, bytes: &[u8], report: &mut ExtractionReport) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, bytes)?;
    report.written.push(path);
    Ok(())
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn slice(bytes: &[u8], start: usize, len: usize) -> io::Result<&[u8]> {
    bytes
        .get(
            start
                ..start
                    .checked_add(len)
                    .ok_or_else(|| invalid("ZIP length overflow"))?,
        )
        .ok_or_else(|| invalid("Truncated ZIP"))
}
fn u16_at(bytes: &[u8], offset: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(
        slice(bytes, offset, 2)?.try_into().unwrap(),
    ))
}
fn u32_at(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        slice(bytes, offset, 4)?.try_into().unwrap(),
    ))
}
struct JarEntry {
    name: String,
    method: u16,
    flags: u16,
    crc: u32,
    packed: usize,
    unpacked: usize,
    local: usize,
}
fn jar_entries(bytes: &[u8]) -> io::Result<Vec<JarEntry>> {
    // Locate EOCD, validating its comment length to exclude signatures in comments.
    let end = (bytes.len().saturating_sub(65557)..bytes.len().saturating_sub(21))
        .rev()
        .find(|&i| {
            u32_at(bytes, i).ok() == Some(0x06054b50)
                && u16_at(bytes, i + 20)
                    .ok()
                    .is_some_and(|n| i + 22 + usize::from(n) == bytes.len())
        })
        .ok_or_else(|| invalid("Missing ZIP end directory"))?;
    if u16_at(bytes, end + 4)? != 0 || u16_at(bytes, end + 6)? != 0 {
        return Err(invalid("Split ZIP archives are unsupported"));
    }

    let eocd_count = u16_at(bytes, end + 10)?;
    if u16_at(bytes, end + 8)? != eocd_count {
        return Err(invalid("Split ZIP archives are unsupported"));
    }
    let eocd_size = u32_at(bytes, end + 12)?;
    let eocd_offset = u32_at(bytes, end + 16)?;
    let has_zip64_locator = end >= 20 && u32_at(bytes, end - 20).ok() == Some(0x07064b50);
    let (count, central_size, central_offset) = if has_zip64_locator {
        let locator = end - 20;
        if u32_at(bytes, locator + 4)? != 0 || u32_at(bytes, locator + 16)? != 1 {
            return Err(invalid("Split ZIP64 archives are unsupported"));
        }
        let record = usize::try_from(u64_at(bytes, locator + 8)?)
            .map_err(|_| invalid("ZIP64 end directory is too large"))?;
        if u32_at(bytes, record)? != 0x06064b50 || u64_at(bytes, record + 4)? < 44 {
            return Err(invalid("Invalid ZIP64 end directory"));
        }
        if u32_at(bytes, record + 16)? != 0 || u32_at(bytes, record + 20)? != 0 {
            return Err(invalid("Split ZIP64 archives are unsupported"));
        }
        let entries_on_disk = u64_at(bytes, record + 24)?;
        let entries_total = u64_at(bytes, record + 32)?;
        if entries_on_disk != entries_total {
            return Err(invalid("Split ZIP64 archives are unsupported"));
        }
        (
            usize::try_from(entries_total).map_err(|_| invalid("Too many ZIP entries"))?,
            usize::try_from(u64_at(bytes, record + 40)?)
                .map_err(|_| invalid("ZIP64 central directory is too large"))?,
            usize::try_from(u64_at(bytes, record + 48)?)
                .map_err(|_| invalid("ZIP64 central directory offset is too large"))?,
        )
    } else {
        if eocd_count == u16::MAX || eocd_size == u32::MAX || eocd_offset == u32::MAX {
            return Err(invalid("Missing ZIP64 end directory locator"));
        }
        (
            usize::from(eocd_count),
            eocd_size as usize,
            eocd_offset as usize,
        )
    };

    let central_end = central_offset
        .checked_add(central_size)
        .ok_or_else(|| invalid("ZIP central directory length overflow"))?;
    slice(bytes, central_offset, central_size)?;
    let mut cursor = central_offset;
    if count > central_size / 46 {
        return Err(invalid("ZIP entry count exceeds central directory size"));
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        if u32_at(bytes, cursor)? != 0x02014b50 {
            return Err(invalid("Invalid ZIP central directory"));
        }
        let flags = u16_at(bytes, cursor + 8)?;
        let method = u16_at(bytes, cursor + 10)?;
        let crc = u32_at(bytes, cursor + 16)?;
        let packed32 = u32_at(bytes, cursor + 20)?;
        let unpacked32 = u32_at(bytes, cursor + 24)?;
        let name_len = usize::from(u16_at(bytes, cursor + 28)?);
        let extra_len = usize::from(u16_at(bytes, cursor + 30)?);
        let comment_len = usize::from(u16_at(bytes, cursor + 32)?);
        let disk16 = u16_at(bytes, cursor + 34)?;
        let local32 = u32_at(bytes, cursor + 42)?;
        let name_start = cursor
            .checked_add(46)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        let extra_start = name_start
            .checked_add(name_len)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        let extra_end = extra_start
            .checked_add(extra_len)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        let entry_end = extra_end
            .checked_add(comment_len)
            .ok_or_else(|| invalid("ZIP offset overflow"))?;
        if entry_end > central_end {
            return Err(invalid("Truncated ZIP central directory"));
        }
        let name = std::str::from_utf8(slice(bytes, name_start, name_len)?)
            .map_err(|_| invalid("Non-UTF8 JAR path"))?
            .to_owned();

        let mut packed = (packed32 != u32::MAX).then_some(u64::from(packed32));
        let mut unpacked = (unpacked32 != u32::MAX).then_some(u64::from(unpacked32));
        let mut local = (local32 != u32::MAX).then_some(u64::from(local32));
        let mut disk = (disk16 != u16::MAX).then_some(u32::from(disk16));
        let mut extra_cursor = extra_start;
        while extra_cursor < extra_end {
            if extra_end - extra_cursor < 4 {
                return Err(invalid("Truncated ZIP extra field"));
            }
            let tag = u16_at(bytes, extra_cursor)?;
            let size = usize::from(u16_at(bytes, extra_cursor + 2)?);
            let data_start = extra_cursor + 4;
            let data_end = data_start
                .checked_add(size)
                .ok_or_else(|| invalid("ZIP extra field overflow"))?;
            if data_end > extra_end {
                return Err(invalid("Truncated ZIP extra field"));
            }
            if tag == 0x0001 {
                let field = slice(bytes, data_start, size)?;
                let mut field_cursor = 0;
                if unpacked.is_none() {
                    unpacked = Some(take_u64(field, &mut field_cursor)?);
                }
                if packed.is_none() {
                    packed = Some(take_u64(field, &mut field_cursor)?);
                }
                if local.is_none() {
                    local = Some(take_u64(field, &mut field_cursor)?);
                }
                if disk.is_none() {
                    disk = Some(take_u32(field, &mut field_cursor)?);
                }
                break;
            }
            extra_cursor = data_end;
        }
        if disk.unwrap_or(0) != 0 {
            return Err(invalid("Split ZIP archives are unsupported"));
        }
        entries.push(JarEntry {
            name,
            flags,
            method,
            crc,
            packed: usize::try_from(packed.ok_or_else(|| invalid("Missing ZIP64 packed size"))?)
                .map_err(|_| invalid("ZIP entry is too large"))?,
            unpacked: usize::try_from(
                unpacked.ok_or_else(|| invalid("Missing ZIP64 unpacked size"))?,
            )
            .map_err(|_| invalid("ZIP entry is too large"))?,
            local: usize::try_from(
                local.ok_or_else(|| invalid("Missing ZIP64 local header offset"))?,
            )
            .map_err(|_| invalid("ZIP entry offset is too large"))?,
        });
        cursor = entry_end;
    }
    Ok(entries)
}
fn take_u64(bytes: &[u8], cursor: &mut usize) -> io::Result<u64> {
    let value = u64::from_le_bytes(slice(bytes, *cursor, 8)?.try_into().unwrap());
    *cursor = cursor
        .checked_add(8)
        .ok_or_else(|| invalid("ZIP64 extra field overflow"))?;
    Ok(value)
}
fn take_u32(bytes: &[u8], cursor: &mut usize) -> io::Result<u32> {
    let value = u32::from_le_bytes(slice(bytes, *cursor, 4)?.try_into().unwrap());
    *cursor = cursor
        .checked_add(4)
        .ok_or_else(|| invalid("ZIP64 extra field overflow"))?;
    Ok(value)
}
fn u64_at(bytes: &[u8], offset: usize) -> io::Result<u64> {
    Ok(u64::from_le_bytes(
        slice(bytes, offset, 8)?.try_into().unwrap(),
    ))
}
impl JarEntry {
    fn decode(&self, archive: &[u8]) -> io::Result<Vec<u8>> {
        if self.flags & 1 != 0 {
            return Err(invalid("Encrypted JAR entry"));
        }
        if u32_at(archive, self.local)? != 0x04034b50 {
            return Err(invalid("Invalid ZIP local header"));
        }
        let start = self.local
            + 30
            + usize::from(u16_at(archive, self.local + 26)?)
            + usize::from(u16_at(archive, self.local + 28)?);
        let packed = slice(archive, start, self.packed)?;
        let data = match self.method {
            0 => packed.to_vec(),
            8 => {
                let mut data = Vec::new();
                flate2::read::DeflateDecoder::new(packed)
                    .take(self.unpacked as u64 + 1)
                    .read_to_end(&mut data)?;
                data
            }
            _ => return Err(invalid("Unsupported ZIP compression method")),
        };
        if data.len() != self.unpacked || crc32fast::hash(&data) != self.crc {
            return Err(invalid("ZIP size or CRC mismatch"));
        }
        Ok(data)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_original_jar_file_decodes_with_valid_length_and_crc() {
        let archive =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../SS2/lib/sinkingsimulator-4.0-all.jar");
        let bytes = fs::read(archive).unwrap();
        let entries = jar_entries(&bytes).unwrap();
        let mut decoded_count = 0;
        for entry in entries {
            if entry.name.ends_with('/') {
                continue;
            }
            let decoded = entry
                .decode(&bytes)
                .unwrap_or_else(|e| panic!("{}: {e}", entry.name));
            assert_eq!(decoded.len(), entry.unpacked);
            decoded_count += 1;
        }
        assert!(decoded_count > 217);
        eprintln!("Verified {decoded_count} original JAR files (length and CRC)");
    }
    #[test]
    fn original_jar_extraction_matches_extracted_resources() {
        let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../SS2/lib");
        let root = std::env::temp_dir().join(format!("ss2-jar-extract-{}", std::process::id()));
        let destination = root.join("icons");
        let report = extract_files_into(
            ResourceSource::Jar(&parent.join("sinkingsimulator-4.0-all.jar")),
            &destination,
            |path| path.extension().is_some_and(|extension| extension == "png"),
        )
        .unwrap();
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(!report.written.is_empty());
        for path in report.written {
            let reference = parent
                .join("sinkingsimulator-4.0-all/icons")
                .join(path.strip_prefix(&destination).unwrap());
            assert_eq!(fs::read(path).unwrap(), fs::read(reference).unwrap());
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn directory_default_filter_reports_directories_but_copies_and_overwrites_files() {
        let root = std::env::temp_dir().join(format!("ss2-dir-extract-{}", std::process::id()));
        let source = root.join("source");
        let destination = root.join("destination");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/test"), b"source").unwrap();
        let first = extract_all(ResourceSource::Directory(&source), &destination).unwrap();
        assert_eq!(first.written.len(), 1);
        assert_eq!(first.errors.len(), 2);
        fs::write(destination.join("nested/test"), b"edited").unwrap();
        let second = extract_files_into(ResourceSource::Directory(&source), &destination, |p| {
            p.is_file()
        })
        .unwrap();
        assert!(second.errors.is_empty());
        assert_eq!(
            fs::read(destination.join("nested/test")).unwrap(),
            b"source"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
