//! TeeOutputStream.java preserves first-output/second-output calls and failure ordering.
#![allow(dead_code)]
use std::io;
pub(crate) trait OutputStream {
    fn write_byte(&mut self, value: i32) -> io::Result<()>;
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn write_range(&mut self, bytes: &[u8], offset: i32, length: i32) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
    fn close(&mut self) -> io::Result<()>;
}
pub(crate) struct TeeOutputStream<A: OutputStream, B: OutputStream> {
    pub out: A,
    pub tee: B,
}
impl<A: OutputStream, B: OutputStream> TeeOutputStream<A, B> {
    pub fn new(out: A, tee: B) -> Self {
        Self { out, tee }
    }
}
impl<A: OutputStream, B: OutputStream> OutputStream for TeeOutputStream<A, B> {
    fn write_byte(&mut self, value: i32) -> io::Result<()> {
        self.out.write_byte(value)?;
        self.tee.write_byte(value)
    }
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.out.write_bytes(bytes)?;
        self.tee.write_bytes(bytes)
    }
    fn write_range(&mut self, bytes: &[u8], offset: i32, length: i32) -> io::Result<()> {
        self.out.write_range(bytes, offset, length)?;
        self.tee.write_range(bytes, offset, length)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()?;
        self.tee.flush()
    }
    fn close(&mut self) -> io::Result<()> {
        self.out.close()?;
        self.tee.close()
    }
}
impl<A: OutputStream, B: OutputStream> io::Write for TeeOutputStream<A, B> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        OutputStream::write_bytes(self, bytes)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        OutputStream::flush(self)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    struct Sink {
        name: &'static str,
        log: Arc<Mutex<Vec<String>>>,
        fail: bool,
    }
    impl Sink {
        fn event(&self, s: String) -> io::Result<()> {
            self.log.lock().unwrap().push(format!("{}:{s}", self.name));
            if self.fail {
                Err(io::Error::other("source sink failure"))
            } else {
                Ok(())
            }
        }
    }
    impl OutputStream for Sink {
        fn write_byte(&mut self, v: i32) -> io::Result<()> {
            self.event(format!("byte:{v}"))
        }
        fn write_bytes(&mut self, b: &[u8]) -> io::Result<()> {
            self.event(format!("bytes:{b:?}"))
        }
        fn write_range(&mut self, b: &[u8], off: i32, len: i32) -> io::Result<()> {
            self.event(format!("range:{b:?}:{off}:{len}"))
        }
        fn flush(&mut self) -> io::Result<()> {
            self.event("flush".into())
        }
        fn close(&mut self) -> io::Result<()> {
            self.event("close".into())
        }
    }
    #[test]
    fn every_operation_calls_original_output_before_tee_and_forwards_arguments() {
        let log = Arc::new(Mutex::new(vec![]));
        let mut stream = TeeOutputStream::new(
            Sink {
                name: "out",
                log: log.clone(),
                fail: false,
            },
            Sink {
                name: "tee",
                log: log.clone(),
                fail: false,
            },
        );
        stream.write_byte(511).unwrap();
        stream.write_bytes(&[]).unwrap();
        stream.write_range(&[1, 2, 3], 1, 2).unwrap();
        OutputStream::flush(&mut stream).unwrap();
        stream.close().unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            [
                "out:byte:511",
                "tee:byte:511",
                "out:bytes:[]",
                "tee:bytes:[]",
                "out:range:[1, 2, 3]:1:2",
                "tee:range:[1, 2, 3]:1:2",
                "out:flush",
                "tee:flush",
                "out:close",
                "tee:close"
            ]
        );
    }
    #[test]
    fn first_failure_prevents_second_call_and_second_failure_keeps_first_side_effect() {
        let log = Arc::new(Mutex::new(vec![]));
        let mut stream = TeeOutputStream::new(
            Sink {
                name: "out",
                log: log.clone(),
                fail: true,
            },
            Sink {
                name: "tee",
                log: log.clone(),
                fail: false,
            },
        );
        assert!(stream.close().is_err());
        assert_eq!(*log.lock().unwrap(), ["out:close"]);
        log.lock().unwrap().clear();
        stream.out.fail = false;
        stream.tee.fail = true;
        assert!(stream.write_bytes(&[7]).is_err());
        assert_eq!(*log.lock().unwrap(), ["out:bytes:[7]", "tee:bytes:[7]"]);
    }
}
