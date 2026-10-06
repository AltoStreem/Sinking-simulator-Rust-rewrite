//! ALDevice.java including class-initialized default device identity.
#![allow(dead_code)]
use crate::{
    al_resource::AlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::sync::{Arc, Mutex, OnceLock};
pub(crate) trait AlDeviceBackend: Send {
    fn get_string(&mut self, device: i64, parameter: i32) -> Option<String>;
    fn open_device(&mut self, name: Option<&str>) -> i64;
    fn close_device(&mut self, id: i64) -> bool;
}
pub(crate) struct AlDevice {
    resource: AlResource<i64>,
    _runtime: ResourceRuntime,
}
impl AlDevice {
    fn open(
        name: Option<&str>,
        backend: Arc<Mutex<dyn AlDeviceBackend>>,
        runtime: &ResourceRuntime,
    ) -> Arc<Self> {
        let id = backend.lock().unwrap().open_device(name);
        let resource = AlResource::new(id, &[], runtime, move || {
            let _ = backend.lock().unwrap().close_device(id);
        });
        Arc::new(Self {
            resource,
            _runtime: runtime.clone(),
        })
    }
    pub fn id(&self) -> i64 {
        *self.resource.id()
    }
    pub fn resource(&self) -> ResourceHandle {
        self.resource.resource()
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}
/// One instance models one loaded Java class. Native application uses class().
#[derive(Default)]
pub(crate) struct AlDeviceClass {
    default: OnceLock<Arc<AlDevice>>,
}
impl AlDeviceClass {
    pub fn default_device(
        &self,
        backend: Arc<Mutex<dyn AlDeviceBackend>>,
        runtime: &ResourceRuntime,
    ) -> Arc<AlDevice> {
        self.default
            .get_or_init(|| {
                let name = backend.lock().unwrap().get_string(0, 4100);
                AlDevice::open(name.as_deref(), backend, runtime)
            })
            .clone()
    }
    pub fn new_device(
        &self,
        name: Option<&str>,
        backend: Arc<Mutex<dyn AlDeviceBackend>>,
        runtime: &ResourceRuntime,
    ) -> Arc<AlDevice> {
        self.default_device(backend.clone(), runtime);
        AlDevice::open(name, backend, runtime)
    }
}
static CLASS: OnceLock<AlDeviceClass> = OnceLock::new();
pub(crate) fn class() -> &'static AlDeviceClass {
    CLASS.get_or_init(AlDeviceClass::default)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Backend(Arc<Mutex<Vec<String>>>);
    impl AlDeviceBackend for Backend {
        fn get_string(&mut self, id: i64, p: i32) -> Option<String> {
            self.0.lock().unwrap().push(format!("name:{id}:{p}"));
            None
        }
        fn open_device(&mut self, name: Option<&str>) -> i64 {
            self.0.lock().unwrap().push(format!("open:{name:?}"));
            0
        }
        fn close_device(&mut self, id: i64) -> bool {
            self.0.lock().unwrap().push(format!("close:{id}"));
            false
        }
    }
    #[test]
    fn null_names_zero_ids_and_closed_default_follow_source_without_validation() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = Arc::new(Mutex::new(Backend(log.clone())));
        let runtime = ResourceRuntime::default();
        let class = AlDeviceClass::default();
        let device = class.new_device(None, backend.clone(), &runtime);
        let default = class.default_device(backend.clone(), &runtime);
        assert_eq!(device.id(), 0);
        assert_eq!(
            *log.lock().unwrap(),
            ["name:0:4100", "open:None", "open:None"]
        );
        runtime.close_all();
        assert!(device.freed());
        assert!(default.freed());
        runtime.run_main();
        assert_eq!(&log.lock().unwrap()[3..], ["close:0", "close:0"]);
        assert!(Arc::ptr_eq(
            &default,
            &class.default_device(backend, &runtime)
        ));
        assert_eq!(log.lock().unwrap().len(), 5);
    }
}
