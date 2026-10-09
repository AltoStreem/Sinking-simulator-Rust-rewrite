//! Original LWJGL 3.2.3 STB decoder, called through its raw-address exports.
//! These particular x64 wrappers ignore JNIEnv/class; array overloads do not.
//! ABI recovery: tools/native-vorbis-source-evidence.md.
#![allow(dead_code)]
use std::{
    ffi::c_void,
    path::Path,
    sync::{Arc, OnceLock},
};
type Malloc = unsafe extern "C" fn(usize) -> *mut c_void;
type Calloc = unsafe extern "C" fn(usize, usize) -> *mut c_void;
type Realloc = unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void;
type Free = unsafe extern "C" fn(*mut c_void);
struct Allocator {
    malloc: Malloc,
    calloc: Calloc,
    realloc: Realloc,
    free: Free,
    aligned_alloc: Calloc,
    aligned_free: Free,
    _library: libloading::Library,
}
// STB stores process-wide callback addresses. Retain their backing library for
// the entire process, including after a NativeVorbis instance is dropped.
static ALLOCATOR: OnceLock<Result<Arc<Allocator>, String>> = OnceLock::new();
const NULL: *mut c_void = std::ptr::null_mut();
fn allocator() -> &'static Allocator {
    ALLOCATOR
        .get()
        .and_then(|r| r.as_ref().ok())
        .expect("allocator installed before decoding")
}
unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    unsafe { (allocator().malloc)(size) }
}
unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    unsafe { (allocator().calloc)(count, size) }
}
unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    unsafe { (allocator().realloc)(ptr, size) }
}
unsafe extern "C" fn free(ptr: *mut c_void) {
    unsafe { (allocator().free)(ptr) }
}
unsafe extern "C" fn aligned_alloc(alignment: usize, size: usize) -> *mut c_void {
    unsafe { (allocator().aligned_alloc)(size, alignment) }
}
unsafe extern "C" fn aligned_free(ptr: *mut c_void) {
    unsafe { (allocator().aligned_free)(ptr) }
}
type Memory = unsafe extern "system" fn(*mut c_void, *mut c_void, i64, i32, i64, i64, i64) -> i32;
type Filename = unsafe extern "system" fn(*mut c_void, *mut c_void, i64, i64, i64, i64) -> i32;
type Setup = unsafe extern "system" fn(*mut c_void, *mut c_void, i64, i64, i64, i64, i64, i64);
struct Decoder {
    memory: Memory,
    filename: Filename,
    _library: libloading::Library,
    allocator: Arc<Allocator>,
}
static DECODER: OnceLock<Result<Arc<Decoder>, String>> = OnceLock::new();
#[derive(Clone)]
pub(crate) struct NativeVorbis(Arc<Decoder>);
pub(crate) struct NativePcm {
    raw: *mut i16,
    pub samples: Vec<i16>,
    pub channels: i32,
    pub rate: i32,
    allocator: Arc<Allocator>,
}
// The allocation is exclusively owned; the original CRT allocator supports
// freeing it on another thread. No context or JNIEnv is retained.
unsafe impl Send for NativePcm {}
impl NativePcm {
    pub fn id(&self) -> i64 {
        self.raw as usize as i64
    }
}
impl Drop for NativePcm {
    fn drop(&mut self) {
        unsafe { (self.allocator.free)(self.raw.cast()) }
    }
}
impl NativeVorbis {
    /// Only the source's supplied Windows x64 LWJGL DLLs have the audited ABI.
    pub fn load(stb_path: &Path) -> Result<Self, String> {
        let allocator = ALLOCATOR
            .get_or_init(|| unsafe {
                // Loading lwjgl.dll without JNI_OnLoad crashes in its thread-detach
                // handler (JVM GetEnv at RVA 0x10ae). Replace its static CRT bridge
                // with the OS CRT, retaining C allocation semantics and matched
                // free/realloc ownership. Source configurable jemalloc/debug
                // allocator behavior is a separate, still incomplete translation.
                let library =
                    libloading::Library::new("ucrtbase.dll").map_err(|e| e.to_string())?;
                macro_rules! symbol {
                    ($suffix:literal) => {
                        *library
                            .get(concat!($suffix, "\0").as_bytes())
                            .map_err(|e| e.to_string())?
                    };
                }
                Ok(Arc::new(Allocator {
                    malloc: symbol!("malloc"),
                    calloc: symbol!("calloc"),
                    realloc: symbol!("realloc"),
                    free: symbol!("free"),
                    aligned_alloc: symbol!("_aligned_malloc"),
                    aligned_free: symbol!("_aligned_free"),
                    _library: library,
                }))
            })
            .as_ref()
            .map_err(Clone::clone)?
            .clone();
        let decoder = DECODER
            .get_or_init(|| unsafe {
                let library = libloading::Library::new(stb_path).map_err(|e| e.to_string())?;
                let memory = *library
                    .get::<Memory>(
                        b"Java_org_lwjgl_stb_STBVorbis_nstb_1vorbis_1decode_1memory__JIJJJ\0",
                    )
                    .map_err(|e| e.to_string())?;
                let filename = *library
                    .get::<Filename>(
                        b"Java_org_lwjgl_stb_STBVorbis_nstb_1vorbis_1decode_1filename__JJJJ\0",
                    )
                    .map_err(|e| e.to_string())?;
                let setup = *library
                    .get::<Setup>(b"Java_org_lwjgl_stb_LibSTB_setupMalloc\0")
                    .map_err(|e| e.to_string())?;
                // Install once before publishing the decoder. Rewriting STB's
                // global callbacks while another thread decodes would be a race.
                setup(
                    NULL,
                    NULL,
                    malloc as *const () as i64,
                    calloc as *const () as i64,
                    realloc as *const () as i64,
                    free as *const () as i64,
                    aligned_alloc as *const () as i64,
                    aligned_free as *const () as i64,
                );
                Ok(Arc::new(Decoder {
                    memory,
                    filename,
                    _library: library,
                    allocator,
                }))
            })
            .as_ref()
            .map_err(Clone::clone)?
            .clone();
        Ok(Self(decoder))
    }
    fn decode(&self, invoke: impl FnOnce(i64, i64, i64) -> i32) -> Option<NativePcm> {
        let mut channels = 0i32;
        let mut rate = 0i32;
        let mut raw: *mut i16 = std::ptr::null_mut();
        let frames = invoke(
            &mut channels as *mut i32 as i64,
            &mut rate as *mut i32 as i64,
            &mut raw as *mut *mut i16 as i64,
        );
        if raw.is_null() {
            return None;
        }
        // Own the allocation before validating/copying, so every exit frees it.
        let mut pcm = NativePcm {
            raw,
            samples: Vec::new(),
            channels,
            rate,
            allocator: self.0.allocator.clone(),
        };
        let count = usize::try_from(frames)
            .ok()?
            .checked_mul(usize::try_from(channels).ok()?)?;
        if count > isize::MAX as usize / 2 {
            return None;
        }
        pcm.samples = unsafe { std::slice::from_raw_parts(raw, count).to_vec() };
        Some(pcm)
    }
    pub fn memory(&self, bytes: &[u8]) -> Option<NativePcm> {
        let length = i32::try_from(bytes.len()).ok()?;
        self.decode(|channels, rate, output| unsafe {
            (self.0.memory)(
                NULL,
                NULL,
                bytes.as_ptr() as i64,
                length,
                channels,
                rate,
                output,
            )
        })
    }
    pub fn filename(&self, path: &str) -> Option<NativePcm> {
        // The source nUTF8 encoder permits NULs; C consumes the prefix.
        let mut bytes = path.as_bytes().to_vec();
        bytes.push(0);
        self.decode(|channels, rate, output| unsafe {
            (self.0.filename)(NULL, NULL, bytes.as_ptr() as i64, channels, rate, output)
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires original Windows x64 STB DLL and soundtrack assets"]
    fn original_native_vorbis_soundtrack_memory_matches_filename() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let native = root.join("../SS2/lib/sinkingsimulator-4.0-all/windows/x64/org/lwjgl");
        let decoder = NativeVorbis::load(&native.join("stb/lwjgl_stb.dll")).unwrap();
        let mut files: Vec<_> = std::fs::read_dir(root.join("assets/music"))
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "ogg"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 10, "Verify the complete included soundtrack");
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../tools/fixtures/source-vorbis-track-pcm.json"
        ))
        .unwrap();
        let reference_tracks = reference["tracks"].as_array().unwrap();
        assert_eq!(reference_tracks.len(), files.len());
        for path in files {
            let bytes = std::fs::read(&path).unwrap();
            let memory = decoder.memory(&bytes).expect("original memory decode");
            let filename = decoder
                .filename(&path.to_string_lossy())
                .expect("original filename decode");
            assert_eq!(
                (memory.channels, memory.rate),
                (filename.channels, filename.rate)
            );
            assert_eq!(memory.samples, filename.samples);
            assert!(memory.channels > 0 && memory.rate > 0 && !memory.samples.is_empty());
            let mut hash = crc32fast::Hasher::new();
            for sample in &memory.samples {
                hash.update(&sample.to_le_bytes());
            }
            let crc = hash.finalize();
            let name = path.file_name().unwrap().to_string_lossy();
            let expected = reference_tracks
                .iter()
                .find(|track| track["file"].as_str() == Some(&name))
                .expect("track present in original native reference");
            assert_eq!(
                memory.channels as i64,
                expected["channels"].as_i64().unwrap()
            );
            assert_eq!(memory.rate as i64, expected["rate"].as_i64().unwrap());
            assert_eq!(
                (memory.samples.len() / memory.channels as usize) as u64,
                expected["frames"].as_u64().unwrap()
            );
            assert_eq!(
                crc,
                u32::from_str_radix(expected["pcm_crc32"].as_str().unwrap(), 16).unwrap(),
                "Original static-CRT decoder PCM fingerprint for {name}"
            );
            println!(
                "{}: channels={} rate={} frames={} pcm_crc32={:08x}",
                path.file_name().unwrap().to_string_lossy(),
                memory.channels,
                memory.rate,
                memory.samples.len() / memory.channels as usize,
                crc
            );
        }
        println!("Testing empty input");
        assert!(decoder.memory(&[]).is_none());
        println!("Testing malformed input");
        assert!(decoder.memory(b"Not an Ogg stream").is_none());
        println!("Testing missing file");
        assert!(
            decoder
                .filename("source-file-that-does-not-exist.ogg")
                .is_none()
        );
    }
}
