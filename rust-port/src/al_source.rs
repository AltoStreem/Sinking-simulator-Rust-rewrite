//! ALSource.java buffer queueing, controls and direct native property queries.
#![allow(dead_code)]
use crate::{al_buffer::AlBuffer, al_resource::AlResource, resource::ResourceRuntime};
use std::sync::{Arc, Mutex};
pub(crate) trait AlSourceBackend: Send {
    fn generate_source(&mut self) -> i32;
    fn delete_source(&mut self, id: i32);
    fn source_integer(&mut self, id: i32, parameter: i32) -> i32;
    fn source_float(&mut self, id: i32, parameter: i32) -> f32;
    fn set_integer(&mut self, id: i32, parameter: i32, value: i32);
    fn set_float(&mut self, id: i32, parameter: i32, value: f32);
    fn queue_buffer(&mut self, id: i32, buffer: i32);
    fn queue_buffers(&mut self, id: i32, buffers: &[i32]);
    fn unqueue_buffer(&mut self, id: i32) -> i32;
    fn play(&mut self, id: i32);
    fn pause(&mut self, id: i32);
    fn stop(&mut self, id: i32);
}
pub(crate) struct AlSource {
    resource: AlResource<i32>,
    backend: Arc<Mutex<dyn AlSourceBackend>>,
}
impl AlSource {
    pub fn new(backend: Arc<Mutex<dyn AlSourceBackend>>, runtime: &ResourceRuntime) -> Self {
        let id = backend.lock().unwrap().generate_source();
        let cleanup = backend.clone();
        Self {
            resource: AlResource::new(id, &[], runtime, move || {
                cleanup.lock().unwrap().delete_source(id)
            }),
            backend,
        }
    }
    pub fn id(&self) -> i32 {
        *self.resource.id()
    }
    pub fn set_buffer(&self, buffer: &AlBuffer) {
        self.backend
            .lock()
            .unwrap()
            .set_integer(self.id(), 4105, buffer.id());
    }
    pub fn queue_buffer(&self, buffer: &AlBuffer) {
        self.backend
            .lock()
            .unwrap()
            .queue_buffer(self.id(), buffer.id());
    }
    pub fn queue_buffers<'a>(&self, buffers: impl IntoIterator<Item = &'a AlBuffer>) {
        let ids: Vec<_> = buffers.into_iter().map(AlBuffer::id).collect();
        self.backend.lock().unwrap().queue_buffers(self.id(), &ids);
    }
    pub fn unqueue_buffer(&self) {
        let _ = self.backend.lock().unwrap().unqueue_buffer(self.id());
    }
    pub fn unqueue_finished_buffers(&self) {
        while self.buffers_processed() > 0 {
            self.unqueue_buffer();
        }
    }
    pub fn play(&self) {
        self.backend.lock().unwrap().play(self.id());
    }
    pub fn pause(&self) {
        self.backend.lock().unwrap().pause(self.id());
    }
    pub fn stop(&self) {
        self.backend.lock().unwrap().stop(self.id());
    }
    pub fn sample_offset(&self) -> i32 {
        self.backend.lock().unwrap().source_integer(self.id(), 4133)
    }
    pub fn seconds_offset(&self) -> f32 {
        self.backend.lock().unwrap().source_float(self.id(), 4132)
    }
    pub fn set_seconds_offset(&self, value: f32) {
        self.backend
            .lock()
            .unwrap()
            .set_float(self.id(), 4132, value);
    }
    pub fn state(&self) -> i32 {
        self.backend.lock().unwrap().source_integer(self.id(), 4112)
    }
    pub fn playing(&self) -> bool {
        self.state() == 4114
    }
    pub fn paused(&self) -> bool {
        self.state() == 4115
    }
    pub fn initial(&self) -> bool {
        self.state() == 4113
    }
    pub fn stopped(&self) -> bool {
        self.state() == 4116
    }
    pub fn buffers_processed(&self) -> i32 {
        self.backend.lock().unwrap().source_integer(self.id(), 4118)
    }
    pub fn volume(&self) -> f32 {
        self.backend.lock().unwrap().source_float(self.id(), 4106)
    }
    pub fn set_volume(&self, value: f32) {
        self.backend
            .lock()
            .unwrap()
            .set_float(self.id(), 4106, value);
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
    struct Backend {
        log: Arc<Mutex<Vec<String>>>,
        processed: i32,
        state: i32,
    }
    impl AlSourceBackend for Backend {
        fn generate_source(&mut self) -> i32 {
            self.log.lock().unwrap().push("generate-source".into());
            3
        }
        fn delete_source(&mut self, id: i32) {
            self.log.lock().unwrap().push(format!("delete-source:{id}"));
        }
        fn source_integer(&mut self, _: i32, p: i32) -> i32 {
            self.log.lock().unwrap().push(format!("integer:{p}"));
            match p {
                4112 => self.state,
                4118 => self.processed,
                4133 => 100,
                _ => panic!("parameter"),
            }
        }
        fn source_float(&mut self, _: i32, p: i32) -> f32 {
            self.log.lock().unwrap().push(format!("float:{p}"));
            0.25
        }
        fn set_integer(&mut self, id: i32, p: i32, v: i32) {
            self.log.lock().unwrap().push(format!("set-i:{id}:{p}:{v}"));
        }
        fn set_float(&mut self, id: i32, p: i32, v: f32) {
            self.log
                .lock()
                .unwrap()
                .push(format!("set-f:{id}:{p}:{}", v.to_bits()));
        }
        fn queue_buffer(&mut self, id: i32, b: i32) {
            self.log.lock().unwrap().push(format!("queue:{id}:{b}"));
        }
        fn queue_buffers(&mut self, id: i32, b: &[i32]) {
            self.log
                .lock()
                .unwrap()
                .push(format!("queue-all:{id}:{b:?}"));
        }
        fn unqueue_buffer(&mut self, id: i32) -> i32 {
            self.processed -= 1;
            self.log.lock().unwrap().push(format!("unqueue:{id}"));
            7
        }
        fn play(&mut self, id: i32) {
            self.log.lock().unwrap().push(format!("play:{id}"));
        }
        fn pause(&mut self, id: i32) {
            self.log.lock().unwrap().push(format!("pause:{id}"));
        }
        fn stop(&mut self, id: i32) {
            self.log.lock().unwrap().push(format!("stop:{id}"));
        }
    }
    #[test]
    fn queue_control_query_order_and_no_buffer_dependency() {
        let runtime = ResourceRuntime::default();
        let (buffer, buffer_backend, _) = crate::al_buffer::tests::fixture(&runtime);
        let second = AlBuffer::new(buffer_backend, &runtime);
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = Arc::new(Mutex::new(Backend {
            log: log.clone(),
            processed: 2,
            state: 4114,
        }));
        let source = AlSource::new(backend.clone(), &runtime);
        source.set_buffer(&buffer);
        source.queue_buffer(&buffer);
        source.queue_buffers([&buffer, &second, &buffer]);
        source.queue_buffers([]);
        source.play();
        source.pause();
        source.stop();
        source.unqueue_finished_buffers();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "generate-source",
                "set-i:3:4105:7",
                "queue:3:7",
                "queue-all:3:[7, 8, 7]",
                "queue-all:3:[]",
                "play:3",
                "pause:3",
                "stop:3",
                "integer:4118",
                "unqueue:3",
                "integer:4118",
                "unqueue:3",
                "integer:4118"
            ]
        );
        assert!(source.playing());
        backend.lock().unwrap().state = 4115;
        assert!(source.paused());
        backend.lock().unwrap().state = 4113;
        assert!(source.initial());
        backend.lock().unwrap().state = 4116;
        assert!(source.stopped());
        assert_eq!(source.sample_offset(), 100);
        assert_eq!(source.seconds_offset(), 0.25);
        assert_eq!(source.volume(), 0.25);
        source.set_seconds_offset(-1.);
        source.set_volume(f32::from_bits(0x7fc00001));
        let records = log.lock().unwrap();
        assert_eq!(
            &records[records.len() - 2..],
            [
                format!("set-f:3:4132:{}", (-1_f32).to_bits()),
                "set-f:3:4106:2143289345".into()
            ]
        );
        drop(records);
        buffer.close();
        assert!(!source.freed());
        source.close();
        runtime.run_main();
        assert_eq!(log.lock().unwrap().last().unwrap(), "delete-source:3");
    }
}
