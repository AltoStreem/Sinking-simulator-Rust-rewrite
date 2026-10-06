//! FileReader.java resolution order; extracted assets replace JVM classpath streams.
use crate::image_data::ImageData;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub(crate) struct FileReader {
    pub ss_home: PathBuf,
    pub ss_resources: PathBuf,
    pub bundled_resources: PathBuf,
}
impl FileReader {
    pub fn new(ss_home: PathBuf, ss_resources: PathBuf, bundled_resources: PathBuf) -> Self {
        // Java mkdirs ignores the boolean result; file opens report any actual error.
        let _ = fs::create_dir_all(&ss_home);
        Self {
            ss_home,
            ss_resources,
            bundled_resources,
        }
    }
    pub fn game() -> Self {
        let user_home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_default();
        let bundled = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .map(|p| p.join("assets"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
        Self::new(
            user_home.join("Sinking Simulator"),
            PathBuf::from("src/main/resources"),
            bundled,
        )
    }
    pub fn as_input_stream(&self, path: &Path) -> io::Result<File> {
        for candidate in [
            path.to_path_buf(),
            self.ss_home.join(path),
            self.ss_resources.join(path),
        ] {
            if candidate.exists() {
                return File::open(candidate);
            }
        }
        // Java classpath lookup normalizes Windows separators and prepends '/'.
        let resource = path.to_string_lossy().replace('\\', "/");
        File::open(
            self.bundled_resources
                .join(resource.trim_start_matches('/')),
        )
    }
    pub fn read_file_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.as_input_stream(path)?.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
    pub fn read_file(&self, path: &Path) -> io::Result<String> {
        // UTF-8 is the original game's modern platform default; malformed input
        // uses replacement characters like InputStreamReader, not a strict error.
        Ok(String::from_utf8_lossy(&self.read_file_bytes(path)?).into_owned())
    }
    pub fn read_image(&self, path: &Path, desired_channels: u8) -> Result<ImageData, String> {
        let bytes = self
            .read_file_bytes(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Self::read_image_stream(&bytes[..], desired_channels)
    }
    pub fn read_image_stream(
        mut stream: impl Read,
        desired_channels: u8,
    ) -> Result<ImageData, String> {
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        let decoded =
            image::load_from_memory(&bytes).map_err(|e| format!("Failed to load image: {e}"))?;
        let channels = if desired_channels == 0 {
            decoded.color().channel_count()
        } else {
            desired_channels
        };
        let (buffer, image_format) = match channels {
            1 => (decoded.to_luma8().into_raw(), 6403),
            2 => (decoded.to_luma_alpha8().into_raw(), 33319),
            3 => (decoded.to_rgb8().into_raw(), 6407),
            4 => (decoded.to_rgba8().into_raw(), 6408),
            _ => return Err(format!("Unsupported image channel count: {channels}")),
        };
        Ok(ImageData::new(
            buffer,
            decoded.width() as i32,
            decoded.height() as i32,
            image_format,
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolution_preserves_source_priority_and_open_errors() {
        let root = std::env::temp_dir().join(format!("ss2-file-reader-{}", std::process::id()));
        let reader = FileReader::new(
            root.join("home"),
            root.join("development"),
            root.join("bundle"),
        );
        for dir in [&reader.ss_resources, &reader.bundled_resources] {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(reader.bundled_resources.join("palette.json"), b"bundled").unwrap();
        assert_eq!(
            reader.read_file(Path::new("palette.json")).unwrap(),
            "bundled"
        );
        fs::write(reader.ss_resources.join("palette.json"), b"development").unwrap();
        assert_eq!(
            reader.read_file(Path::new("palette.json")).unwrap(),
            "development"
        );
        fs::write(reader.ss_home.join("palette.json"), b"home").unwrap();
        assert_eq!(reader.read_file(Path::new("palette.json")).unwrap(), "home");
        let direct = root.join("direct.json");
        fs::write(&direct, b"direct").unwrap();
        assert_eq!(reader.read_file(&direct).unwrap(), "direct");
        assert!(reader.as_input_stream(Path::new("missing")).is_err());
        // An existing directory must fail opening, never fall through to a lower priority file.
        fs::create_dir_all(reader.ss_home.join("directory")).unwrap();
        fs::write(reader.bundled_resources.join("directory"), b"wrong").unwrap();
        assert!(reader.read_file(Path::new("directory")).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn png_rgba_preserves_pixels_and_source_metadata() {
        let pixels = image::RgbaImage::from_pixel(3, 2, image::Rgba([19, 71, 159, 191]));
        let mut encoded = std::io::Cursor::new(Vec::new());
        pixels
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let data = FileReader::read_image_stream(&encoded.into_inner()[..], 4).unwrap();
        assert_eq!(
            (data.width, data.height, data.image_format, data.format),
            (3, 2, 6408, 5121)
        );
        assert_eq!(data.into_rgba().unwrap(), pixels);
    }
}
