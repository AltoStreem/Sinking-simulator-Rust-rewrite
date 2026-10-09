//! ALContext.java: global current pointer, lazy capabilities and device dependency.
#![allow(dead_code)]
use crate::{
    al_device::AlDevice,
    al_resource::AlResource,
    resource::{ResourceHandle, ResourceRuntime},
};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
pub(crate) type CapabilityObject = Arc<dyn Any + Send + Sync>;
pub(crate) trait AlContextBackend: Send {
    fn create_context(&mut self, device: i64, attributes: &[i32]) -> i64;
    fn make_current(&mut self, context: i64) -> bool;
    fn create_alc_capabilities(&mut self, id: i64) -> CapabilityObject;
    fn create_al_capabilities(&mut self, alc: &CapabilityObject) -> CapabilityObject;
    fn try_alc_capabilities(&mut self, id: i64) -> Result<CapabilityObject, String> {
        Ok(self.create_alc_capabilities(id))
    }
    fn try_al_capabilities(&mut self, alc: &CapabilityObject) -> Result<CapabilityObject, String> {
        Ok(self.create_al_capabilities(alc))
    }
    fn destroy_context(&mut self, id: i64);
}
pub(crate) struct AlContext {
    resource: AlResource<i64>,
    pub device: Arc<AlDevice>,
    backend: Arc<Mutex<dyn AlContextBackend>>,
    alc_capabilities: Mutex<Option<CapabilityObject>>,
    al_capabilities: Mutex<Option<CapabilityObject>>,
}
static CURRENT: Mutex<Option<Arc<AlContext>>> = Mutex::new(None);
impl AlContext {
    pub fn new_default(
        device_backend: Arc<Mutex<dyn crate::al_device::AlDeviceBackend>>,
        backend: Arc<Mutex<dyn AlContextBackend>>,
        runtime: &ResourceRuntime,
    ) -> Arc<Self> {
        Self::new(
            crate::al_device::class().default_device(device_backend, runtime),
            backend,
            runtime,
        )
    }
    pub fn new(
        device: Arc<AlDevice>,
        backend: Arc<Mutex<dyn AlContextBackend>>,
        runtime: &ResourceRuntime,
    ) -> Arc<Self> {
        let id = backend.lock().unwrap().create_context(device.id(), &[0]);
        let cleanup = backend.clone();
        let resource = AlResource::new(id, &[device.resource()], runtime, move || {
            cleanup.lock().unwrap().destroy_context(id)
        });
        Arc::new(Self {
            resource,
            device,
            backend,
            alc_capabilities: Mutex::new(None),
            al_capabilities: Mutex::new(None),
        })
    }
    pub fn id(&self) -> i64 {
        *self.resource.id()
    }
    pub fn resource(&self) -> ResourceHandle {
        self.resource.resource()
    }
    pub fn current() -> Option<Arc<Self>> {
        CURRENT.lock().unwrap().clone()
    }
    pub fn start(self: &Arc<Self>) {
        self.try_start().unwrap_or_else(|error| panic!("{error}"));
    }
    /// Same source start ordering, with native exceptions represented as errors.
    /// A failed AL constructor leaves current and the initialized ALC property
    /// assigned, exactly as the original exception path does.
    pub fn try_start(self: &Arc<Self>) -> Result<(), String> {
        let _ = self.backend.lock().unwrap().make_current(self.id());
        *CURRENT.lock().unwrap() = Some(self.clone());
        if self.alc_capabilities.lock().unwrap().is_none() {
            let alc = self
                .backend
                .lock()
                .unwrap()
                .try_alc_capabilities(self.id())?;
            *self.alc_capabilities.lock().unwrap() = Some(alc.clone());
            let al = self.backend.lock().unwrap().try_al_capabilities(&alc)?;
            *self.al_capabilities.lock().unwrap() = Some(al);
        }
        Ok(())
    }
    pub fn stop(&self) {
        let _ = self.backend.lock().unwrap().make_current(0);
        *CURRENT.lock().unwrap() = None;
    }
    pub fn alc_capabilities(&self) -> CapabilityObject {
        let value = self.alc_capabilities.lock().unwrap().clone();
        value.expect("lateinit property alcCapabilities has not been initialized")
    }
    pub fn set_alc_capabilities(&self, value: CapabilityObject) {
        *self.alc_capabilities.lock().unwrap() = Some(value);
    }
    pub fn al_capabilities(&self) -> CapabilityObject {
        let value = self.al_capabilities.lock().unwrap().clone();
        value.expect("lateinit property alCapabilities has not been initialized")
    }
    pub fn set_al_capabilities(&self, value: CapabilityObject) {
        *self.al_capabilities.lock().unwrap() = Some(value);
    }
    pub fn close(&self) {
        self.resource.close();
    }
    pub fn freed(&self) -> bool {
        self.resource.freed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Both fixtures exercise the same source process-global CURRENT pointer.
    // Serialize their complete lifetimes, including the cross-thread read.
    static CURRENT_FIXTURE: Mutex<()> = Mutex::new(());
    use crate::{
        al_context_start_reference::AlcCapabilitiesReference,
        al_device::{AlDeviceBackend, AlDeviceClass},
    };
    type Log = Arc<Mutex<Vec<String>>>;
    struct DeviceBackend {
        log: Log,
        next: i64,
    }
    impl AlDeviceBackend for DeviceBackend {
        fn get_string(&mut self, id: i64, p: i32) -> Option<String> {
            self.log.lock().unwrap().push(format!("name:{id}:{p}"));
            Some("default".into())
        }
        fn open_device(&mut self, name: Option<&str>) -> i64 {
            self.next += 1;
            self.log.lock().unwrap().push(format!("open:{name:?}"));
            self.next
        }
        fn close_device(&mut self, id: i64) -> bool {
            self.log.lock().unwrap().push(format!("close:{id}"));
            false
        }
    }
    struct Backend {
        log: Log,
        next: i64,
        failure: Option<&'static str>,
    }
    impl AlContextBackend for Backend {
        fn try_alc_capabilities(&mut self, id: i64) -> Result<CapabilityObject, String> {
            if self.failure == Some("ALC") {
                self.log.lock().unwrap().push(format!("alc-error:{id}"));
                Err("ALC constructor failed".into())
            } else {
                Ok(self.create_alc_capabilities(id))
            }
        }
        fn try_al_capabilities(
            &mut self,
            alc: &CapabilityObject,
        ) -> Result<CapabilityObject, String> {
            if self.failure == Some("AL") {
                self.log.lock().unwrap().push("al-error".into());
                Err("AL constructor failed".into())
            } else {
                Ok(self.create_al_capabilities(alc))
            }
        }
        fn create_context(&mut self, id: i64, attributes: &[i32]) -> i64 {
            self.next += 1;
            self.log
                .lock()
                .unwrap()
                .push(format!("create:{id}:{attributes:?}"));
            self.next
        }
        fn make_current(&mut self, id: i64) -> bool {
            self.log.lock().unwrap().push(format!("current:{id}"));
            false
        }
        fn create_alc_capabilities(&mut self, id: i64) -> CapabilityObject {
            self.log.lock().unwrap().push(format!("alc:{id}"));
            Arc::new(id)
        }
        fn create_al_capabilities(&mut self, alc: &CapabilityObject) -> CapabilityObject {
            self.log
                .lock()
                .unwrap()
                .push(format!("al:{}", alc.downcast_ref::<i64>().unwrap()));
            Arc::new(7_u32)
        }
        fn destroy_context(&mut self, id: i64) {
            self.log.lock().unwrap().push(format!("destroy:{id}"));
        }
    }
    #[test]
    fn failed_start_retains_source_current_and_partial_lazy_properties() {
        let _fixture = CURRENT_FIXTURE.lock().unwrap();
        let runtime = ResourceRuntime::default();
        let log = Log::default();
        let class = AlDeviceClass::default();
        let devices = Arc::new(Mutex::new(DeviceBackend {
            log: log.clone(),
            next: 40,
        }));
        let device = class.default_device(devices, &runtime);
        let backend = Arc::new(Mutex::new(Backend {
            log: log.clone(),
            next: 99,
            failure: Some("ALC"),
        }));
        let context = AlContext::new(device.clone(), backend.clone(), &runtime);
        assert_eq!(context.try_start().unwrap_err(), "ALC constructor failed");
        assert!(Arc::ptr_eq(&context, &AlContext::current().unwrap()));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| context.alc_capabilities()))
                .is_err()
        );
        backend.lock().unwrap().failure = Some("AL");
        assert_eq!(context.try_start().unwrap_err(), "AL constructor failed");
        let alc = context.alc_capabilities();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| context.al_capabilities()))
                .is_err()
        );
        backend.lock().unwrap().failure = None;
        context.try_start().unwrap();
        assert!(Arc::ptr_eq(&alc, &context.alc_capabilities()));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| context.al_capabilities()))
                .is_err(),
            "cached ALC skips the failed AL constructor on restart"
        );
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|line| *line == "al-error")
                .count(),
            1
        );
        context.stop();
        device.close();
        runtime.run_main();
        assert!(context.freed() && device.freed());
    }
    #[test]
    fn initialization_global_current_lazy_capabilities_property_reference_and_cleanup() {
        let _fixture = CURRENT_FIXTURE.lock().unwrap();
        let runtime = ResourceRuntime::default();
        let log = Log::default();
        let class = AlDeviceClass::default();
        let devices = Arc::new(Mutex::new(DeviceBackend {
            log: log.clone(),
            next: 40,
        }));
        let device = class.new_device(Some("named"), devices.clone(), &runtime);
        let default = class.default_device(devices.clone(), &runtime);
        assert!(Arc::ptr_eq(
            &default,
            &class.default_device(devices, &runtime)
        ));
        assert_eq!(
            *log.lock().unwrap(),
            [
                "name:0:4100",
                "open:Some(\"default\")",
                "open:Some(\"named\")"
            ]
        );
        let backend = Arc::new(Mutex::new(Backend {
            log: log.clone(),
            next: 99,
            failure: None,
        }));
        let context = AlContext::new(device.clone(), backend.clone(), &runtime);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| context.alc_capabilities()))
                .is_err()
        );
        context.start();
        context.start();
        assert_eq!(
            &log.lock().unwrap()[3..],
            [
                "create:42:[0]",
                "current:100",
                "alc:100",
                "al:100",
                "current:100"
            ]
        );
        let caps = context.alc_capabilities();
        assert_eq!(*caps.downcast_ref::<i64>().unwrap(), 100);
        assert_eq!(*context.al_capabilities().downcast_ref::<u32>().unwrap(), 7);
        let cross_thread = std::thread::spawn(|| AlContext::current().unwrap())
            .join()
            .unwrap();
        assert!(Arc::ptr_eq(&context, &cross_thread));
        context.close();
        assert!(Arc::ptr_eq(&context, &AlContext::current().unwrap()));
        runtime.run_main();
        assert_eq!(log.lock().unwrap().last().unwrap(), "destroy:100");
        let other = AlContext::new(device.clone(), backend, &runtime);
        let reference = AlcCapabilitiesReference {
            receiver: other.clone(),
        };
        assert_eq!(reference.name(), "alcCapabilities");
        assert_eq!(
            reference.signature(),
            "getAlcCapabilities()Lorg/lwjgl/openal/ALCCapabilities;"
        );
        let supplied: CapabilityObject = Arc::new(55_i64);
        reference.set(Some(supplied.clone()));
        assert!(Arc::ptr_eq(&reference.get(), &supplied));
        other.start();
        assert!(Arc::ptr_eq(&other, &AlContext::current().unwrap()));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| other.al_capabilities()))
                .is_err()
        );
        assert_eq!(log.lock().unwrap().last().unwrap(), "current:101");
        context.stop();
        assert!(AlContext::current().is_none());
        device.close();
        assert!(other.freed());
        runtime.run_main();
        let records = log.lock().unwrap();
        assert_eq!(&records[records.len() - 2..], ["destroy:101", "close:42"]);
    }
}
