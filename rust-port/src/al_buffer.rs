//! ALBuffer.java native allocation, Vorbis upload and metadata arithmetic.
#![allow(dead_code)]
use crate::{al_resource::AlResource, mem_util::NativeBuffer, resource::ResourceRuntime};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
pub(crate) struct DecodedPcm {
    pub id: i64,
    pub samples: Vec<i16>,
}
pub(crate) trait AlBufferBackend: Send {
    fn generate_buffer(&mut self) -> i32;
    fn delete_buffer(&mut self, id: i32);
    fn buffer_integer(&mut self, id: i32, parameter: i32) -> i32;
    fn stack_push(&mut self);
    fn stack_malloc_int(&mut self) -> i64;
    fn stack_pop(&mut self);
    fn read_int(&mut self, slot: i64) -> i32;
    fn decode_memory(
        &mut self,
        bytes: &[u8],
        channels: i64,
        sample_rate: i64,
    ) -> Option<DecodedPcm>;
    fn decode_filename(
        &mut self,
        path: &str,
        channels: i64,
        sample_rate: i64,
    ) -> Option<DecodedPcm>;
    fn buffer_data(&mut self, id: i32, format: i32, samples: &[i16], sample_rate: i32);
    fn free_pcm(&mut self, id: i64);
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UnsupportedExtension(pub String);
impl std::fmt::Display for UnsupportedExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unsupported extension: {}", self.0)
    }
}
impl std::error::Error for UnsupportedExtension {}
pub(crate) struct AlBuffer {
    resource: AlResource<i32>,
    backend: Arc<Mutex<dyn AlBufferBackend>>,
}
impl AlBuffer {
    pub fn new(backend: Arc<Mutex<dyn AlBufferBackend>>, runtime: &ResourceRuntime) -> Self {
        let id = backend.lock().unwrap().generate_buffer();
        let cleanup = backend.clone();
        Self {
            resource: AlResource::new(id, &[], runtime, move || {
                cleanup.lock().unwrap().delete_buffer(id)
            }),
            backend,
        }
    }
    pub fn from_file(
        path: &Path,
        backend: Arc<Mutex<dyn AlBufferBackend>>,
        runtime: &ResourceRuntime,
    ) -> Result<Self, UnsupportedExtension> {
        let buffer = Self::new(backend, runtime);
        buffer.load_file(path)?;
        Ok(buffer)
    }
    pub fn id(&self) -> i32 {
        *self.resource.id()
    }
    pub fn load_file(&self, path: &Path) -> Result<(), UnsupportedExtension> {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let extension = name
            .rsplit_once('.')
            .map(|(_, extension)| extension)
            .unwrap_or("");
        if extension != "ogg" {
            return Err(UnsupportedExtension(extension.into()));
        }
        self.load_vorbis_filename(&path.to_string_lossy());
        Ok(())
    }
    pub fn load_vorbis_memory(&self, buffer: &NativeBuffer<u8>) {
        self.decode(|backend, channels, rate| {
            backend.decode_memory(buffer.remaining(), channels, rate)
        });
    }
    pub fn load_vorbis_filename(&self, path: &str) {
        self.decode(|backend, channels, rate| backend.decode_filename(path, channels, rate));
    }
    fn decode(
        &self,
        decoder: impl FnOnce(&mut dyn AlBufferBackend, i64, i64) -> Option<DecodedPcm>,
    ) {
        let mut backend = self.backend.lock().unwrap();
        backend.stack_push();
        let channels_slot = backend.stack_malloc_int();
        backend.stack_push();
        let rate_slot = backend.stack_malloc_int();
        let raw = decoder(&mut *backend, channels_slot, rate_slot);
        let channels = backend.read_int(channels_slot);
        let rate = backend.read_int(rate_slot);
        backend.stack_pop();
        backend.stack_pop();
        let format = match channels {
            1 => 4353,
            2 => 4355,
            _ => -1,
        };
        if let Some(raw) = raw {
            backend.buffer_data(self.id(), format, &raw.samples, rate);
            backend.free_pcm(raw.id);
        }
    }
    fn integer(&self, parameter: i32) -> i32 {
        self.backend
            .lock()
            .unwrap()
            .buffer_integer(self.id(), parameter)
    }
    pub fn size(&self) -> i32 {
        self.integer(8196)
    }
    pub fn channels(&self) -> i32 {
        self.integer(8195)
    }
    pub fn bits(&self) -> i32 {
        self.integer(8194)
    }
    pub fn frequency(&self) -> i32 {
        self.integer(8193)
    }
    pub fn samples(&self) -> i32 {
        if self.channels().wrapping_mul(self.bits()) > 0 {
            self.size()
                .wrapping_mul(8)
                .wrapping_div(self.channels().wrapping_mul(self.bits()))
        } else {
            0
        }
    }
    pub fn duration(&self) -> f32 {
        self.samples() as f32 / self.frequency() as f32
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) type Log = Arc<Mutex<Vec<String>>>;
    pub(crate) struct Backend {
        pub log: Log,
        pub channels: i32,
        pub rate: i32,
        pub size: i32,
        pub decoded: bool,
        slot: i64,
        next_buffer: i32,
    }
    impl AlBufferBackend for Backend {
        fn generate_buffer(&mut self) -> i32 {
            self.log.lock().unwrap().push("generate".into());
            self.next_buffer += 1;
            self.next_buffer
        }
        fn delete_buffer(&mut self, id: i32) {
            self.log.lock().unwrap().push(format!("delete:{id}"));
        }
        fn buffer_integer(&mut self, _: i32, p: i32) -> i32 {
            self.log.lock().unwrap().push(format!("query:{p}"));
            match p {
                8195 => self.channels,
                8194 => 16,
                8196 => self.size,
                8193 => self.rate,
                _ => panic!("parameter"),
            }
        }
        fn stack_push(&mut self) {
            self.log.lock().unwrap().push("push".into());
        }
        fn stack_malloc_int(&mut self) -> i64 {
            self.slot += 1;
            self.log
                .lock()
                .unwrap()
                .push(format!("malloc:{}", self.slot));
            self.slot
        }
        fn stack_pop(&mut self) {
            self.log.lock().unwrap().push("pop".into());
        }
        fn read_int(&mut self, slot: i64) -> i32 {
            self.log.lock().unwrap().push(format!("read:{slot}"));
            if slot % 2 == 1 {
                self.channels
            } else {
                self.rate
            }
        }
        fn decode_memory(&mut self, bytes: &[u8], c: i64, r: i64) -> Option<DecodedPcm> {
            self.log
                .lock()
                .unwrap()
                .push(format!("memory:{bytes:?}:{c}:{r}"));
            self.decoded.then(|| DecodedPcm {
                id: 99,
                samples: vec![1, -2],
            })
        }
        fn decode_filename(&mut self, path: &str, c: i64, r: i64) -> Option<DecodedPcm> {
            self.log
                .lock()
                .unwrap()
                .push(format!("file:{path}:{c}:{r}"));
            self.decoded.then(|| DecodedPcm {
                id: 99,
                samples: vec![1, -2],
            })
        }
        fn buffer_data(&mut self, id: i32, f: i32, s: &[i16], r: i32) {
            self.log
                .lock()
                .unwrap()
                .push(format!("upload:{id}:{f}:{s:?}:{r}"));
        }
        fn free_pcm(&mut self, id: i64) {
            self.log.lock().unwrap().push(format!("free:{id}"));
        }
    }
    pub(crate) fn fixture(runtime: &ResourceRuntime) -> (AlBuffer, Arc<Mutex<Backend>>, Log) {
        let log = Log::default();
        let backend = Arc::new(Mutex::new(Backend {
            log: log.clone(),
            channels: 2,
            rate: 24,
            size: 48,
            decoded: true,
            slot: 0,
            next_buffer: 6,
        }));
        (AlBuffer::new(backend.clone(), runtime), backend, log)
    }
    #[test]
    fn decode_uses_remaining_input_stack_order_formats_and_explicit_free() {
        for (channels, decoded, format) in [
            (1, true, 4353),
            (2, true, 4355),
            (3, true, -1),
            (2, false, 4355),
        ] {
            let runtime = ResourceRuntime::default();
            let (buffer, backend, log) = fixture(&runtime);
            backend.lock().unwrap().channels = channels;
            backend.lock().unwrap().decoded = decoded;
            let mut bytes = crate::mem_util::wrap_byte_buffer(&[9, 10, 11, 12]);
            bytes.set_position(1).unwrap();
            bytes.set_limit(3).unwrap();
            buffer.load_vorbis_memory(&bytes);
            let mut expected: Vec<String> = [
                "generate",
                "push",
                "malloc:1",
                "push",
                "malloc:2",
                "memory:[10, 11]:1:2",
                "read:1",
                "read:2",
                "pop",
                "pop",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect();
            if decoded {
                expected.extend([format!("upload:7:{format}:[1, -2]:24"), "free:99".into()]);
            }
            assert_eq!(*log.lock().unwrap(), expected);
            assert_eq!(bytes.position(), 1);
        }
    }
    #[test]
    fn extension_queries_duration_and_integer_overflow_match_source() {
        let runtime = ResourceRuntime::default();
        let (buffer, backend, log) = fixture(&runtime);
        assert_eq!(
            buffer
                .load_file(Path::new("track.OGG"))
                .unwrap_err()
                .to_string(),
            "Unsupported extension: OGG"
        );
        assert_eq!(
            buffer
                .load_file(Path::new("track"))
                .unwrap_err()
                .to_string(),
            "Unsupported extension: "
        );
        buffer.load_file(Path::new(".ogg")).unwrap();
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|line| line == "file:.ogg:1:2")
        );
        log.lock().unwrap().clear();
        assert_eq!(buffer.duration(), 0.5);
        assert_eq!(
            *log.lock().unwrap(),
            [
                "query:8195",
                "query:8194",
                "query:8196",
                "query:8195",
                "query:8194",
                "query:8193"
            ]
        );
        backend.lock().unwrap().size = i32::MAX;
        assert_eq!(buffer.samples(), 0);
        backend.lock().unwrap().channels = 0;
        backend.lock().unwrap().rate = 0;
        assert!(buffer.duration().is_nan());
        buffer.close();
        assert!(buffer.freed());
        runtime.run_main();
        assert_eq!(log.lock().unwrap().last().unwrap(), "delete:7");
    }
}
