//! Main.java's timestamp, logs.txt and sequential System.out/System.err replacement.
//! JVM PrintStream encoding/error swallowing and process-wide Rust stream wiring remain pending.
use std::{
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
};
pub(crate) trait Operations {
    type Stream: Clone;
    fn current_time_millis(&mut self) -> i64;
    fn open_log(&mut self, path: &Path) -> Result<Self::Stream, String>;
    fn stdout(&mut self) -> Option<Self::Stream>;
    fn stderr(&mut self) -> Option<Self::Stream>;
    fn print_stream(&mut self, original: Self::Stream, log: Self::Stream) -> Self::Stream;
    fn set_stdout(&mut self, stream: Self::Stream);
    fn set_stderr(&mut self, stream: Self::Stream);
}
pub(crate) fn prepare(ops: &mut impl Operations) -> Result<i64, String> {
    let base_time = ops.current_time_millis();
    let log = ops.open_log(Path::new("logs.txt"))?;
    let original = ops.stdout().ok_or("System.out must not be null")?;
    let print = ops.print_stream(original, log.clone());
    ops.set_stdout(print);
    // Source reads stderr only after publishing the new stdout.
    let original = ops.stderr().ok_or("System.err must not be null")?;
    let print = ops.print_stream(original, log);
    ops.set_stderr(print);
    Ok(base_time)
}
/// Both TeeOutputStream instances retain the same FileOutputStream.
#[derive(Clone)]
pub(crate) struct SharedLog(Arc<Mutex<Option<std::fs::File>>>);
impl SharedLog {
    pub fn open(path: &Path) -> io::Result<Self> {
        // FileOutputStream(File) truncates; it does not create parent directories.
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map(|file| Self(Arc::new(Mutex::new(Some(file)))))
    }
    fn with_file<T>(
        &self,
        call: impl FnOnce(&mut std::fs::File) -> io::Result<T>,
    ) -> io::Result<T> {
        let mut file = self.0.lock().unwrap();
        call(
            file.as_mut()
                .ok_or_else(|| io::Error::other("Stream Closed"))?,
        )
    }
}
impl crate::tee_output_stream::OutputStream for SharedLog {
    fn write_byte(&mut self, value: i32) -> io::Result<()> {
        self.with_file(|file| file.write_all(&[value as u8]))
    }
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.with_file(|file| file.write_all(bytes))
    }
    fn write_range(&mut self, bytes: &[u8], offset: i32, length: i32) -> io::Result<()> {
        let end = offset
            .checked_add(length)
            .filter(|end| offset >= 0 && length >= 0 && (*end as usize) <= bytes.len())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "IndexOutOfBoundsException")
            })?;
        self.write_bytes(&bytes[offset as usize..end as usize])
    }
    fn flush(&mut self) -> io::Result<()> {
        self.with_file(|file| file.flush())
    }
    fn close(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().take();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Ops {
        events: Vec<String>,
        out: Option<usize>,
        err: Option<usize>,
        fail_open: bool,
    }
    impl Operations for Ops {
        type Stream = usize;
        fn current_time_millis(&mut self) -> i64 {
            self.events.push("clock".into());
            123
        }
        fn open_log(&mut self, path: &Path) -> Result<usize, String> {
            self.events.push(format!("open:{}", path.display()));
            if self.fail_open {
                Err("open failed".into())
            } else {
                Ok(7)
            }
        }
        fn stdout(&mut self) -> Option<usize> {
            self.events.push("getOut".into());
            self.out
        }
        fn stderr(&mut self) -> Option<usize> {
            assert_eq!(self.out, Some(17));
            self.events.push("getErr".into());
            self.err
        }
        fn print_stream(&mut self, original: usize, log: usize) -> usize {
            self.events.push(format!("tee/print:{original}:{log}"));
            original + log + 9
        }
        fn set_stdout(&mut self, stream: usize) {
            self.events.push("setOut".into());
            self.out = Some(stream);
        }
        fn set_stderr(&mut self, stream: usize) {
            self.events.push("setErr".into());
            self.err = Some(stream);
        }
    }
    fn fixture() -> Ops {
        Ops {
            events: vec![],
            out: Some(1),
            err: Some(2),
            fail_open: false,
        }
    }
    #[test]
    fn shared_log_identity_and_sequential_stream_lookup_follow_source() {
        let mut ops = fixture();
        assert_eq!(prepare(&mut ops), Ok(123));
        assert_eq!(
            ops.events,
            [
                "clock",
                "open:logs.txt",
                "getOut",
                "tee/print:1:7",
                "setOut",
                "getErr",
                "tee/print:2:7",
                "setErr"
            ]
        );
        assert_eq!((ops.out, ops.err), (Some(17), Some(18)));
    }
    #[test]
    fn failures_preserve_original_partial_installation() {
        let mut ops = fixture();
        ops.fail_open = true;
        assert_eq!(prepare(&mut ops), Err("open failed".into()));
        assert_eq!(ops.events, ["clock", "open:logs.txt"]);
        let mut ops = fixture();
        ops.out = None;
        assert!(prepare(&mut ops).unwrap_err().contains("System.out"));
        assert_eq!(ops.events.len(), 3);
        let mut ops = fixture();
        ops.err = None;
        assert!(prepare(&mut ops).unwrap_err().contains("System.err"));
        assert_eq!(ops.out, Some(17));
        assert_eq!(ops.err, None);
        assert_eq!(ops.events.last().unwrap(), "getErr");
    }
    struct Console(Vec<u8>);
    impl crate::tee_output_stream::OutputStream for Console {
        fn write_byte(&mut self, v: i32) -> io::Result<()> {
            self.0.push(v as u8);
            Ok(())
        }
        fn write_bytes(&mut self, b: &[u8]) -> io::Result<()> {
            self.0.extend_from_slice(b);
            Ok(())
        }
        fn write_range(&mut self, b: &[u8], off: i32, len: i32) -> io::Result<()> {
            self.write_bytes(&b[off as usize..(off + len) as usize])
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn close(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn converted_tees_share_truncated_file_and_close_affects_both() {
        use crate::tee_output_stream::{OutputStream, TeeOutputStream};
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("ss2-main-log-{}-{stamp}.txt", std::process::id()));
        std::fs::write(&path, b"old contents").unwrap();
        let log = SharedLog::open(&path).unwrap();
        let mut out = TeeOutputStream::new(Console(vec![]), log.clone());
        let mut err = TeeOutputStream::new(Console(vec![]), log);
        out.write_bytes(b"out").unwrap();
        err.write_range(b"_err_", 1, 3).unwrap();
        out.write_byte(511).unwrap();
        OutputStream::flush(&mut out).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"outerr\xff");
        assert_eq!(out.out.0, b"out\xff");
        assert_eq!(err.out.0, b"err");
        out.close().unwrap();
        assert!(err.write_bytes(b"x").is_err());
        assert_eq!(err.out.0, b"errx"); // first tee output succeeds before closed file fails
        err.close().unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
