//! Concrete native OpenAL device/source calls for the translated interfaces.
//! Uses the source's supplied OpenAL implementation, without JNI or a JVM.
#![allow(dead_code)]
use crate::{al_device::AlDeviceBackend, al_source::AlSourceBackend, al_util_kt::AlErrorBackend};
use std::{
    ffi::{CStr, c_char, c_void},
    path::Path,
    sync::Arc,
};
type Handle = *mut c_void;
macro_rules! api {
    ($(fn $name:ident($($arg:ty),*) -> $result:ty;)+) => {
        #[allow(non_snake_case)]
        struct Api { $($name:unsafe extern "C" fn($($arg),*) -> $result,)+ }
        impl Api {
            unsafe fn load(library:&libloading::Library)->Result<Self,String> {
                Ok(Self { $($name:unsafe { *library.get(concat!(stringify!($name),"\0").as_bytes())
                    .map_err(|error|error.to_string())? },)+ })
            }
        }
    }
}
api! {
    fn alcGetString(Handle,i32)->*const c_char;
    fn alcOpenDevice(*const c_char)->Handle;
    fn alcCloseDevice(Handle)->i8;
    fn alcCreateContext(Handle,*const i32)->Handle;
    fn alcMakeContextCurrent(Handle)->i8;
    fn alcGetCurrentContext()->Handle;
    fn alcDestroyContext(Handle)->();
    fn alcGetError(Handle)->i32;
    fn alcGetIntegerv(Handle,i32,i32,*mut i32)->();
    fn alcIsExtensionPresent(Handle,*const c_char)->i8;
    fn alcGetProcAddress(Handle,*const c_char)->Handle;
    fn alGetString(i32)->*const c_char;
    fn alIsExtensionPresent(*const c_char)->i8;
    fn alGetProcAddress(*const c_char)->Handle;
    fn alGenSources(i32,*mut u32)->();
    fn alDeleteSources(i32,*const u32)->();
    fn alIsSource(u32)->i8;
    fn alGetSourcei(u32,i32,*mut i32)->();
    fn alGetSourcef(u32,i32,*mut f32)->();
    fn alSourcei(u32,i32,i32)->();
    fn alSourcef(u32,i32,f32)->();
    fn alSourceQueueBuffers(u32,i32,*const u32)->();
    fn alSourceUnqueueBuffers(u32,i32,*mut u32)->();
    fn alSourcePlay(u32)->();
    fn alSourcePause(u32)->();
    fn alSourceStop(u32)->();
    fn alGenBuffers(i32,*mut u32)->();
    fn alDeleteBuffers(i32,*const u32)->();
    fn alIsBuffer(u32)->i8;
    fn alGetBufferi(u32,i32,*mut i32)->();
    fn alBufferData(u32,i32,*const c_void,i32,i32)->();
    fn alGetError()->i32;
}
struct Library {
    api: Api,
    _library: libloading::Library,
}
#[derive(Clone)]
pub(crate) struct NativeOpenAl(Arc<Library>);
/// Retain the provider for as long as any capability address is reachable.
pub(crate) struct NativeAlcCapabilities {
    pub(crate) id: i64,
    pub(crate) version: (i32, i32),
    pub(crate) entries: crate::al_capabilities::Capabilities,
    _provider: NativeOpenAl,
}
pub(crate) struct NativeAlCapabilities {
    pub(crate) version: (i32, i32),
    pub(crate) version_string: String,
    pub(crate) entries: crate::al_capabilities::Capabilities,
    _provider: NativeOpenAl,
}
static PROCESS_CAPABILITIES: std::sync::Mutex<Option<Arc<NativeAlCapabilities>>> =
    std::sync::Mutex::new(None);
thread_local! {
    static THREAD_CAPABILITIES: std::cell::RefCell<Option<Arc<NativeAlCapabilities>>> =
        const { std::cell::RefCell::new(None) };
}
pub(crate) fn set_process_capabilities(value: Option<Arc<NativeAlCapabilities>>) {
    *PROCESS_CAPABILITIES.lock().unwrap() = value;
    THREAD_CAPABILITIES.with(|caps| *caps.borrow_mut() = None);
}
pub(crate) fn set_thread_capabilities(value: Option<Arc<NativeAlCapabilities>>) {
    THREAD_CAPABILITIES.with(|caps| *caps.borrow_mut() = value);
}
pub(crate) fn current_capabilities() -> Result<Arc<NativeAlCapabilities>, String> {
    THREAD_CAPABILITIES.with(|caps| caps.borrow().clone())
        .or_else(|| PROCESS_CAPABILITIES.lock().unwrap().clone())
        .ok_or_else(|| "No ALCapabilities instance set for the current thread or process. Possible solutions:\n\ta) Call AL.createCapabilities() after making a context current.\n\tb) Call AL.setCurrentProcess() or AL.setCurrentThread() if an ALCapabilities instance already exists.".into())
}
impl NativeOpenAl {
    pub(crate) fn current_context_id(&self) -> i64 {
        unsafe { (self.0.api.alcGetCurrentContext)() as usize as i64 }
    }
    fn direct_address(&self, name: &str) -> usize {
        let name = std::ffi::CString::new(name).unwrap();
        // SAFETY: lookup only. The retained provider owns the returned address;
        // zero reproduces LWJGL's missing-function result.
        unsafe {
            self.0
                ._library
                .get::<unsafe extern "C" fn()>(name.as_bytes_with_nul())
                .map_or(0, |entry| *entry as usize)
        }
    }
    fn al_address(&self, name: &str) -> usize {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe { (self.0.api.alGetProcAddress)(name.as_ptr()) as usize }
    }
    pub(crate) fn alc_capabilities(&self, id: i64) -> Result<Arc<NativeAlcCapabilities>, String> {
        use crate::{
            al_capabilities::{Capabilities, extension_tokens, version_extensions},
            al_capability_tables::{ALC_FLAGS, ALC_SYMBOLS},
        };
        for name in ["alcGetIntegerv", "alcGetString", "alcIsExtensionPresent"] {
            if self.direct_address(name) == 0 {
                return Err(
                    "Core ALC functions could not be found. Make sure that OpenAL has been loaded."
                        .into(),
                );
            }
        }
        let handle = id as usize as Handle;
        // Source reuses a single stack int for both queries. The original DLL
        // writes it for device, context and null IDs (native probe evidence).
        let mut slot = 0;
        unsafe {
            (self.0.api.alcGetIntegerv)(handle, 4096, 1, &mut slot);
        }
        let major = slot;
        unsafe {
            (self.0.api.alcGetIntegerv)(handle, 4097, 1, &mut slot);
        }
        let version = (major, slot);
        let mut reported = version_extensions("OpenALC", version.0, version.1);
        let text = unsafe { (self.0.api.alcGetString)(handle, 4102) };
        if !text.is_null() {
            let text = unsafe { CStr::from_ptr(text) }.to_string_lossy();
            for extension in extension_tokens(&text) {
                let name = std::ffi::CString::new(extension).unwrap();
                if unsafe { (self.0.api.alcIsExtensionPresent)(handle, name.as_ptr()) } != 0 {
                    reported.insert(extension.to_owned());
                }
            }
        }
        let entries = Capabilities::new(ALC_SYMBOLS, ALC_FLAGS, &reported, |symbol| {
            if symbol.device {
                let name = std::ffi::CString::new(symbol.name).unwrap();
                unsafe { (self.0.api.alcGetProcAddress)(handle, name.as_ptr()) as usize }
            } else {
                self.direct_address(symbol.name)
            }
        });
        for extension in &entries.missing {
            eprintln!("[ALC] {extension} was reported as available but an entry point is missing.");
        }
        Ok(Arc::new(NativeAlcCapabilities {
            id,
            version,
            entries,
            _provider: self.clone(),
        }))
    }
    pub(crate) fn al_capabilities(
        &self,
        alc: &NativeAlcCapabilities,
    ) -> Result<Arc<NativeAlCapabilities>, String> {
        use crate::{
            al_capabilities::{Capabilities, extension_tokens, parse_version, version_extensions},
            al_capability_tables::{AL_FLAGS, AL_SYMBOLS},
        };
        let result = (|| {
            let get_string = self.al_address("alGetString");
            let get_error = self.al_address("alGetError");
            let is_extension = self.al_address("alIsExtensionPresent");
            if get_string == 0 || get_error == 0 || is_extension == 0 {
                return Err("Core OpenAL functions could not be found. Make sure that the OpenAL library has been loaded correctly.".into());
            }
            // SAFETY: the three provider addresses were checked and match
            // their documented C signatures, as in the source JNI invocations.
            let get_string: unsafe extern "C" fn(i32) -> *const c_char =
                unsafe { std::mem::transmute(get_string) };
            let get_error: unsafe extern "C" fn() -> i32 =
                unsafe { std::mem::transmute(get_error) };
            let is_extension: unsafe extern "C" fn(*const c_char) -> i8 =
                unsafe { std::mem::transmute(is_extension) };
            let text = unsafe { get_string(45058) };
            // Preserve Java short circuit: a null version doesn't consume AL error.
            if text.is_null() || unsafe { get_error() } != 0 {
                return Err(
                    "There is no OpenAL context current in the current thread or process.".into(),
                );
            }
            let version_string = unsafe { CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned();
            let version = parse_version(&version_string)?;
            let mut reported = version_extensions("OpenAL", version.0, version.1);
            let text = unsafe { get_string(45060) };
            if !text.is_null() {
                let text = unsafe { CStr::from_ptr(text) }.to_string_lossy();
                for extension in extension_tokens(&text) {
                    let name = std::ffi::CString::new(extension).unwrap();
                    if unsafe { is_extension(name.as_ptr()) } != 0 {
                        reported.insert(extension.to_owned());
                    }
                }
            }
            if alc.entries.enabled("ALC_EXT_EFX") {
                reported.insert("ALC_EXT_EFX".into());
            }
            let entries = Capabilities::new(AL_SYMBOLS, AL_FLAGS, &reported, |symbol| {
                self.al_address(symbol.name)
            });
            for extension in &entries.missing {
                eprintln!(
                    "[AL] {extension} was reported as available but an entry point is missing."
                );
            }
            Ok(Arc::new(NativeAlCapabilities {
                version,
                version_string,
                entries,
                _provider: self.clone(),
            }))
        })();
        // Source publishes even on a failed construction, clearing that slot.
        let value = result.as_ref().ok().cloned();
        let thread_current = if alc.entries.enabled("ALC_EXT_thread_local_context") {
            let address = alc.entries.address("alcGetThreadContext");
            let get: unsafe extern "C" fn() -> Handle = unsafe { std::mem::transmute(address) };
            unsafe { !get().is_null() }
        } else {
            false
        };
        if thread_current {
            set_thread_capabilities(value);
        } else {
            set_process_capabilities(value);
        }
        result
    }
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        // SAFETY: callers supply the original trusted native library. Each
        // required C symbol is loaded with its OpenAL ABI and retained library.
        unsafe {
            let library = libloading::Library::new(path).map_err(|e| e.to_string())?;
            let api = Api::load(&library)?;
            Ok(Self(Arc::new(Library {
                api,
                _library: library,
            })))
        }
    }
    pub(crate) fn context(&self, device: i64) -> Result<NativeContext, String> {
        unsafe {
            let previous = (self.0.api.alcGetCurrentContext)();
            let context = (self.0.api.alcCreateContext)(device as usize as Handle, [0].as_ptr());
            if context.is_null() {
                return Err(format!(
                    "alcCreateContext failed: {}",
                    (self.0.api.alcGetError)(device as usize as Handle)
                ));
            }
            if (self.0.api.alcMakeContextCurrent)(context) == 0 {
                (self.0.api.alcDestroyContext)(context);
                return Err("alcMakeContextCurrent failed".into());
            }
            // ALContext.start passes its context ID to ALC.createCapabilities,
            // even though that API's argument is named device in LWJGL.
            let initialized = self
                .alc_capabilities(context as usize as i64)
                .and_then(|alc| self.al_capabilities(&alc).map(|al| (alc, al)));
            let (alc_capabilities, al_capabilities) = match initialized {
                Ok(caps) => caps,
                Err(error) => {
                    (self.0.api.alcMakeContextCurrent)(previous);
                    (self.0.api.alcDestroyContext)(context);
                    return Err(error);
                }
            };
            Ok(NativeContext {
                backend: self.clone(),
                context,
                previous,
                alc_capabilities,
                al_capabilities,
                _thread: std::marker::PhantomData,
            })
        }
    }
    /// Native PCM upload; source STB Vorbis decoding is not implemented here.
    pub(crate) fn pcm_buffer(
        &mut self,
        channels: i32,
        rate: i32,
        samples: &[i16],
    ) -> Result<i32, String> {
        let format = match channels {
            1 => 4353,
            2 => 4355,
            _ => return Err("Source mono/stereo PCM required".into()),
        };
        let bytes = i32::try_from(std::mem::size_of_val(samples))
            .map_err(|_| "PCM exceeds source ALsizei range")?;
        unsafe {
            let mut id = 0;
            (self.0.api.alGenBuffers)(1, &mut id);
            (self.0.api.alBufferData)(id, format, samples.as_ptr().cast(), bytes, rate);
            let error = (self.0.api.alGetError)();
            if error != 0 {
                (self.0.api.alDeleteBuffers)(1, &id);
                return Err(format!("PCM upload AL error {error}"));
            }
            Ok(id as i32)
        }
    }
    pub(crate) fn buffer_integer(&self, id: i32, parameter: i32) -> i32 {
        unsafe {
            let mut result = 0;
            (self.0.api.alGetBufferi)(id as u32, parameter, &mut result);
            result
        }
    }
    pub(crate) fn delete_buffer(&self, id: i32) {
        unsafe {
            (self.0.api.alDeleteBuffers)(1, &(id as u32));
        }
    }
    pub(crate) fn is_buffer(&self, id: i32) -> bool {
        unsafe { (self.0.api.alIsBuffer)(id as u32) != 0 }
    }
    pub(crate) fn is_source(&self, id: i32) -> bool {
        unsafe { (self.0.api.alIsSource)(id as u32) != 0 }
    }
}
impl crate::al_context::AlContextBackend for NativeOpenAl {
    fn try_alc_capabilities(
        &mut self,
        id: i64,
    ) -> Result<crate::al_context::CapabilityObject, String> {
        self.alc_capabilities(id)
            .map(|caps| caps as crate::al_context::CapabilityObject)
    }
    fn try_al_capabilities(
        &mut self,
        alc: &crate::al_context::CapabilityObject,
    ) -> Result<crate::al_context::CapabilityObject, String> {
        let alc = alc
            .downcast_ref::<NativeAlcCapabilities>()
            .expect("ALCCapabilities required");
        self.al_capabilities(alc)
            .map(|caps| caps as crate::al_context::CapabilityObject)
    }
    fn create_context(&mut self, device: i64, attributes: &[i32]) -> i64 {
        assert!(
            attributes.contains(&0),
            "native ALC attributes require a terminator"
        );
        unsafe {
            (self.0.api.alcCreateContext)(device as usize as Handle, attributes.as_ptr()) as usize
                as i64
        }
    }
    fn make_current(&mut self, context: i64) -> bool {
        unsafe { (self.0.api.alcMakeContextCurrent)(context as usize as Handle) != 0 }
    }
    fn create_alc_capabilities(&mut self, id: i64) -> crate::al_context::CapabilityObject {
        self.alc_capabilities(id)
            .unwrap_or_else(|error| panic!("{error}"))
    }
    fn create_al_capabilities(
        &mut self,
        alc: &crate::al_context::CapabilityObject,
    ) -> crate::al_context::CapabilityObject {
        let alc = alc
            .downcast_ref::<NativeAlcCapabilities>()
            .expect("ALCCapabilities required");
        self.al_capabilities(alc)
            .unwrap_or_else(|error| panic!("{error}"))
    }
    fn destroy_context(&mut self, id: i64) {
        unsafe {
            (self.0.api.alcDestroyContext)(id as usize as Handle);
        }
    }
}
/// Native context scope is thread-bound. Restore its caller before destroying
/// it; the retained library outlives all loaded function pointers.
pub(crate) struct NativeContext {
    backend: NativeOpenAl,
    context: Handle,
    previous: Handle,
    pub(crate) alc_capabilities: Arc<NativeAlcCapabilities>,
    pub(crate) al_capabilities: Arc<NativeAlCapabilities>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl Drop for NativeContext {
    fn drop(&mut self) {
        unsafe {
            if (self.backend.0.api.alcGetCurrentContext)() == self.context {
                (self.backend.0.api.alcMakeContextCurrent)(self.previous);
            }
            (self.backend.0.api.alcDestroyContext)(self.context);
        }
    }
}
impl AlDeviceBackend for NativeOpenAl {
    fn get_string(&mut self, device: i64, parameter: i32) -> Option<String> {
        unsafe {
            let text = (self.0.api.alcGetString)(device as usize as Handle, parameter);
            (!text.is_null()).then(|| CStr::from_ptr(text).to_string_lossy().into_owned())
        }
    }
    fn open_device(&mut self, name: Option<&str>) -> i64 {
        // LWJGL nUTF8Safe terminates the string without rejecting an embedded
        // NUL; the native device selector consumes its prefix in that case.
        let name = name.map(|name| {
            let mut bytes = name.as_bytes().to_vec();
            bytes.push(0);
            bytes
        });
        unsafe {
            (self.0.api.alcOpenDevice)(
                name.as_ref()
                    .map_or(std::ptr::null(), |name| name.as_ptr().cast()),
            ) as usize as i64
        }
    }
    fn close_device(&mut self, id: i64) -> bool {
        unsafe { (self.0.api.alcCloseDevice)(id as usize as Handle) != 0 }
    }
}
impl AlErrorBackend for NativeOpenAl {
    fn get_error(&mut self) -> i32 {
        unsafe { (self.0.api.alGetError)() }
    }
}
impl AlSourceBackend for NativeOpenAl {
    fn generate_source(&mut self) -> i32 {
        unsafe {
            let mut id = 0;
            (self.0.api.alGenSources)(1, &mut id);
            id as i32
        }
    }
    fn delete_source(&mut self, id: i32) {
        unsafe {
            (self.0.api.alDeleteSources)(1, &(id as u32));
        }
    }
    fn source_integer(&mut self, id: i32, p: i32) -> i32 {
        unsafe {
            let mut v = 0;
            (self.0.api.alGetSourcei)(id as u32, p, &mut v);
            v
        }
    }
    fn source_float(&mut self, id: i32, p: i32) -> f32 {
        unsafe {
            let mut v = 0.0;
            (self.0.api.alGetSourcef)(id as u32, p, &mut v);
            v
        }
    }
    fn set_integer(&mut self, id: i32, p: i32, v: i32) {
        unsafe {
            (self.0.api.alSourcei)(id as u32, p, v);
        }
    }
    fn set_float(&mut self, id: i32, p: i32, v: f32) {
        unsafe {
            (self.0.api.alSourcef)(id as u32, p, v);
        }
    }
    fn queue_buffer(&mut self, id: i32, buffer: i32) {
        self.queue_buffers(id, &[buffer]);
    }
    fn queue_buffers(&mut self, id: i32, buffers: &[i32]) {
        let ids: Vec<u32> = buffers.iter().map(|&id| id as u32).collect();
        let count = i32::try_from(ids.len()).expect("Source ALsizei queue count");
        unsafe {
            (self.0.api.alSourceQueueBuffers)(id as u32, count, ids.as_ptr());
        }
    }
    fn unqueue_buffer(&mut self, id: i32) -> i32 {
        unsafe {
            let mut buffer = 0;
            (self.0.api.alSourceUnqueueBuffers)(id as u32, 1, &mut buffer);
            buffer as i32
        }
    }
    fn play(&mut self, id: i32) {
        unsafe {
            (self.0.api.alSourcePlay)(id as u32);
        }
    }
    fn pause(&mut self, id: i32) {
        unsafe {
            (self.0.api.alSourcePause)(id as u32);
        }
    }
    fn stop(&mut self, id: i32) {
        unsafe {
            (self.0.api.alSourceStop)(id as u32);
        }
    }
}

/// Concrete ALBuffer backend: the source STB decoder, native AL upload and
/// matching source CRT release. The integer slots model the two nested stack
/// scopes; decoder outputs use local native integers rather than fake addresses.
pub(crate) struct NativeAudioBuffers {
    openal: NativeOpenAl,
    decoder: crate::native_vorbis::NativeVorbis,
    slots: std::collections::HashMap<i64, i32>,
    scopes: Vec<Vec<i64>>,
    next_slot: i64,
    pcm: std::collections::HashMap<i64, crate::native_vorbis::NativePcm>,
}
impl NativeAudioBuffers {
    pub fn new(openal: NativeOpenAl, decoder: crate::native_vorbis::NativeVorbis) -> Self {
        Self {
            openal,
            decoder,
            slots: Default::default(),
            scopes: Vec::new(),
            next_slot: 0,
            pcm: Default::default(),
        }
    }
    fn decoded(
        &mut self,
        pcm: crate::native_vorbis::NativePcm,
        channels: i64,
        rate: i64,
    ) -> crate::al_buffer::DecodedPcm {
        *self.slots.get_mut(&channels).expect("live channel slot") = pcm.channels;
        *self.slots.get_mut(&rate).expect("live sample rate slot") = pcm.rate;
        let mut pcm = pcm;
        let id = pcm.id();
        let samples = std::mem::take(&mut pcm.samples);
        assert!(
            self.pcm.insert(id, pcm).is_none(),
            "native allocation still owned"
        );
        crate::al_buffer::DecodedPcm { id, samples }
    }
}
impl crate::al_buffer::AlBufferBackend for NativeAudioBuffers {
    fn generate_buffer(&mut self) -> i32 {
        let mut id = 0;
        unsafe {
            (self.openal.0.api.alGenBuffers)(1, &mut id);
        }
        id as i32
    }
    fn delete_buffer(&mut self, id: i32) {
        self.openal.delete_buffer(id);
    }
    fn buffer_integer(&mut self, id: i32, parameter: i32) -> i32 {
        self.openal.buffer_integer(id, parameter)
    }
    fn stack_push(&mut self) {
        self.scopes.push(Vec::new());
    }
    fn stack_malloc_int(&mut self) -> i64 {
        self.next_slot += 1;
        self.scopes
            .last_mut()
            .expect("active source stack scope")
            .push(self.next_slot);
        // The source malloc slot is uninitialized on decode failure. Its value
        // is only used for a format when PCM exists; initialize safely here.
        self.slots.insert(self.next_slot, 0);
        self.next_slot
    }
    fn stack_pop(&mut self) {
        for slot in self.scopes.pop().expect("balanced source stack scopes") {
            self.slots.remove(&slot);
        }
    }
    fn read_int(&mut self, slot: i64) -> i32 {
        self.slots[&slot]
    }
    fn decode_memory(
        &mut self,
        bytes: &[u8],
        channels: i64,
        rate: i64,
    ) -> Option<crate::al_buffer::DecodedPcm> {
        let pcm = self.decoder.memory(bytes)?;
        Some(self.decoded(pcm, channels, rate))
    }
    fn decode_filename(
        &mut self,
        path: &str,
        channels: i64,
        rate: i64,
    ) -> Option<crate::al_buffer::DecodedPcm> {
        let pcm = self.decoder.filename(path)?;
        Some(self.decoded(pcm, channels, rate))
    }
    fn buffer_data(&mut self, id: i32, format: i32, samples: &[i16], rate: i32) {
        let bytes =
            i32::try_from(std::mem::size_of_val(samples)).expect("source ALsizei PCM length");
        unsafe {
            (self.openal.0.api.alBufferData)(
                id as u32,
                format,
                samples.as_ptr().cast(),
                bytes,
                rate,
            );
        }
    }
    fn free_pcm(&mut self, id: i64) {
        // Dropping NativePcm calls the exact allocator used by STB.
        drop(self.pcm.remove(&id).expect("source PCM allocation owned"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires original OpenAL DLL and thread-local context extension; no playback"]
    fn native_capabilities_thread_override_process_fallback_and_failed_construction() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../SS2/lib/sinkingsimulator-4.0-all/windows/x64/org/lwjgl/openal/OpenAL.dll");
        let mut native = NativeOpenAl::load(&path).unwrap();
        let device = native.open_device(None);
        assert_ne!(device, 0);
        let context = native.context(device).unwrap();
        let process = current_capabilities().unwrap();
        assert!(Arc::ptr_eq(&process, &context.al_capabilities));
        let alc = &context.alc_capabilities;
        assert!(alc.entries.enabled("ALC_EXT_thread_local_context"));
        let address = alc.entries.address("alcSetThreadContext");
        let set_thread: unsafe extern "C" fn(Handle) -> i8 =
            unsafe { std::mem::transmute(address) };
        assert_ne!(unsafe { set_thread(context.context) }, 0);
        let local = native.al_capabilities(alc).unwrap();
        assert!(!Arc::ptr_eq(&local, &process));
        assert!(Arc::ptr_eq(&current_capabilities().unwrap(), &local));
        let other = std::thread::spawn(current_capabilities)
            .join()
            .unwrap()
            .unwrap();
        assert!(
            Arc::ptr_eq(&other, &process),
            "thread capability doesn't replace process fallback"
        );
        set_thread_capabilities(None);
        assert!(Arc::ptr_eq(&current_capabilities().unwrap(), &process));
        set_thread_capabilities(Some(local));
        set_process_capabilities(Some(process.clone()));
        assert!(
            Arc::ptr_eq(&current_capabilities().unwrap(), &process),
            "process setter clears caller TLS"
        );
        assert_ne!(unsafe { set_thread(std::ptr::null_mut()) }, 0);
        drop(context);
        set_process_capabilities(None);
        assert!(current_capabilities().is_err());
        assert!(native.close_device(device));
        println!(
            "Original native capabilities: actual thread-local context selects TLS, other threads inherit process caps, TLS clearing restores fallback, process setter clears caller TLS"
        );
    }
    #[test]
    #[ignore = "Requires original OpenAL DLL and native output device; no playback"]
    fn native_original_al_context_capabilities_lazy_identity_and_cleanup() {
        use crate::al_context::AlContext;
        use std::sync::Mutex;
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../SS2/lib/sinkingsimulator-4.0-all/windows/x64/org/lwjgl/openal/OpenAL.dll");
        let native = NativeOpenAl::load(&path).unwrap();
        let backend = Arc::new(Mutex::new(native.clone()));
        let runtime = crate::resource::ResourceRuntime::default();
        let class = crate::al_device::AlDeviceClass::default();
        let device = class.default_device(backend.clone(), &runtime);
        assert_ne!(device.id(), 0);
        let context = AlContext::new(device.clone(), backend, &runtime);
        assert_ne!(context.id(), 0);
        context.start();
        let alc = context.alc_capabilities();
        let al = context.al_capabilities();
        let alc_typed = alc.downcast_ref::<NativeAlcCapabilities>().unwrap();
        let al_typed = al.downcast_ref::<NativeAlCapabilities>().unwrap();
        assert_eq!(
            alc_typed.id,
            context.id(),
            "source uses context ID, not device"
        );
        assert_eq!(alc_typed.version, (1, 1));
        assert_eq!(al_typed.version, (1, 1));
        assert!(alc_typed.entries.enabled("OpenALC10") && alc_typed.entries.enabled("OpenALC11"));
        assert!(al_typed.entries.enabled("OpenAL10") && al_typed.entries.enabled("OpenAL11"));
        assert!(alc_typed.entries.missing.is_empty() && al_typed.entries.missing.is_empty());
        let device_caps = native.alc_capabilities(device.id()).unwrap();
        assert!(device_caps.entries.enabled("ALC_EXT_EFX"));
        assert!(!alc_typed.entries.enabled("ALC_EXT_EFX"));
        assert!(!al_typed.entries.enabled("ALC_EXT_EFX"));
        assert_eq!(alc_typed.entries.addresses.len(), 30);
        assert_eq!(al_typed.entries.addresses.len(), 118);
        assert!(Arc::ptr_eq(&context, &AlContext::current().unwrap()));
        context.stop();
        // Context stop clears only ALContext.current, not AL's cached capabilities.
        assert!(AlContext::current().is_none());
        assert!(std::ptr::eq(
            current_capabilities().unwrap().as_ref(),
            al_typed
        ));
        context.start();
        assert!(Arc::ptr_eq(&alc, &context.alc_capabilities()));
        assert!(Arc::ptr_eq(&al, &context.al_capabilities()));
        assert_eq!(
            unsafe { (native.0.api.alcGetCurrentContext)() } as usize as i64,
            context.id()
        );
        context.stop();
        let error = match native.al_capabilities(alc_typed) {
            Ok(_) => panic!("missing current context must fail"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            "There is no OpenAL context current in the current thread or process."
        );
        assert!(
            current_capabilities().is_err(),
            "failed constructor clears process slot"
        );
        context.start(); // already initialized: source does not republish AL capabilities.
        assert!(current_capabilities().is_err());
        context.stop();
        device.close(); // Source dependency closes context before its device.
        assert!(context.freed() && device.freed());
        runtime.run_main();
        assert_eq!(
            unsafe { (native.0.api.alcGetError)(std::ptr::null_mut()) },
            0
        );
        println!(
            "Original native context: 30 ALC/118 AL symbols, context-ID EFX filtering, lazy capability identity, start/stop, missing-context publication and dependency cleanup passed"
        );
    }
    #[test]
    #[ignore = "Requires original OpenAL/STB DLLs, soundtrack and output device"]
    fn native_openal_vorbis_buffers_upload_and_release_all_tracks() {
        use std::sync::Mutex;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let lib = root.join("../SS2/lib/sinkingsimulator-4.0-all/windows/x64/org/lwjgl");
        let mut native = NativeOpenAl::load(&lib.join("openal/OpenAL.dll")).unwrap();
        let decoder =
            crate::native_vorbis::NativeVorbis::load(&lib.join("stb/lwjgl_stb.dll")).unwrap();
        let device = native.open_device(None);
        assert_ne!(device, 0);
        {
            let _context = native.context(device).unwrap();
            let runtime = crate::resource::ResourceRuntime::default();
            let backend = Arc::new(Mutex::new(NativeAudioBuffers::new(
                native.clone(),
                decoder.clone(),
            )));
            let mut files: Vec<_> = std::fs::read_dir(root.join("assets/music"))
                .unwrap()
                .map(|f| f.unwrap().path())
                .filter(|p| p.extension().is_some_and(|e| e == "ogg"))
                .collect();
            files.sort();
            assert_eq!(files.len(), 10);
            for path in files {
                let expected = decoder.filename(&path.to_string_lossy()).unwrap();
                let buffer =
                    crate::al_buffer::AlBuffer::from_file(&path, backend.clone(), &runtime)
                        .unwrap();
                assert_eq!(buffer.channels(), expected.channels);
                assert_eq!(buffer.frequency(), expected.rate);
                assert_eq!(buffer.bits(), 16);
                assert_eq!(buffer.size() as usize, expected.samples.len() * 2);
                assert_eq!(
                    buffer.samples() as usize,
                    expected.samples.len() / expected.channels as usize
                );
                assert_eq!(
                    buffer.duration(),
                    buffer.samples() as f32 / expected.rate as f32
                );
                // Memory upload must consume only ByteBuffer.remaining().
                let bytes = std::fs::read(&path).unwrap();
                let mut padded = b"prefix".to_vec();
                padded.extend_from_slice(&bytes);
                padded.extend_from_slice(b"suffix");
                let mut memory = crate::mem_util::wrap_byte_buffer(&padded);
                memory.set_position(6).unwrap();
                memory.set_limit(6 + bytes.len()).unwrap();
                buffer.load_vorbis_memory(&memory);
                assert_eq!(buffer.size() as usize, expected.samples.len() * 2);
                assert_eq!(buffer.channels(), expected.channels);
                assert_eq!(buffer.frequency(), expected.rate);
                {
                    let state = backend.lock().unwrap();
                    assert!(state.pcm.is_empty(), "PCM freed after each upload");
                    assert!(
                        state.slots.is_empty() && state.scopes.is_empty(),
                        "both nested stack scopes released"
                    );
                }
                crate::al_util_kt::al_check(&mut native).unwrap();
                let id = buffer.id();
                buffer.close();
                runtime.run_main();
                assert!(buffer.freed() && !native.is_buffer(id));
                println!(
                    "{}: memory/file ALBuffer upload, metadata and deferred deletion passed",
                    path.file_name().unwrap().to_string_lossy()
                );
            }
            crate::al_util_kt::al_check(&mut native).unwrap();
        }
        println!("All soundtrack buffers released; native context destroyed");
        assert!(native.close_device(device));
        println!("Native soundtrack output device closed");
        drop(native);
        println!("Native soundtrack OpenAL library scope released");
    }
    #[test]
    #[ignore = "Requires original native OpenAL library and output device; explicitly run with --ignored"]
    fn native_openal_source_controls_queue_offsets_and_cleanup() {
        use std::sync::Mutex;
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../SS2/lib/sinkingsimulator-4.0-all/windows/x64/org/lwjgl/openal/OpenAL.dll");
        let mut native = NativeOpenAl::load(&path).unwrap();
        let runtime = crate::resource::ResourceRuntime::default();
        let backend = Arc::new(Mutex::new(native.clone()));
        let class = crate::al_device::AlDeviceClass::default();
        let device = class.default_device(backend.clone(), &runtime);
        assert_ne!(
            device.id(),
            0,
            "Original OpenAL could not open default output device"
        );
        {
            let _context = native.context(device.id()).unwrap();
            let buffer = native.pcm_buffer(1, 48000, &vec![0i16; 48000]).unwrap();
            assert_eq!(native.buffer_integer(buffer, 8193), 48000);
            assert_eq!(native.buffer_integer(buffer, 8194), 16);
            assert_eq!(native.buffer_integer(buffer, 8195), 1);
            assert_eq!(native.buffer_integer(buffer, 8196), 96000);
            let source = crate::al_source::AlSource::new(backend.clone(), &runtime);
            assert!(source.initial());
            source.set_volume(0.0);
            assert_eq!(source.volume(), 0.0);
            native.set_integer(source.id(), 4105, buffer);
            source.play();
            assert!(source.playing());
            source.pause();
            assert!(source.paused());
            source.set_seconds_offset(0.25);
            assert_eq!(source.seconds_offset(), 0.25);
            assert_eq!(source.sample_offset(), 12000);
            source.stop();
            assert!(source.stopped());
            native.set_integer(source.id(), 4105, 0);
            native.queue_buffers(source.id(), &[buffer, buffer]);
            assert_eq!(native.source_integer(source.id(), 4117), 2);
            source.play();
            source.stop();
            assert_eq!(source.buffers_processed(), 2);
            source.unqueue_finished_buffers();
            assert_eq!(native.source_integer(source.id(), 4117), 0);
            crate::al_util_kt::al_check(&mut native).unwrap();
            let id = source.id();
            source.close();
            runtime.run_main();
            assert!(!native.is_source(id));
            native.delete_buffer(buffer);
            assert!(!native.is_buffer(buffer));
            crate::al_util_kt::al_check(&mut native).unwrap();
        }
        device.close();
        runtime.run_main();
        assert!(device.freed());
        println!(
            "Original native OpenAL: PCM metadata, muted controls, 0.25s/12000-sample seek, two-buffer queue, deferred source/device cleanup passed"
        );
    }
}
