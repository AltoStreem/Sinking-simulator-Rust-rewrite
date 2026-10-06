//! ALUtilKt.java error query and exact source exception messages.
#![allow(dead_code)]
pub(crate) trait AlErrorBackend {
    fn get_error(&mut self) -> i32;
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AlError(pub String);
impl std::fmt::Display for AlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for AlError {}
pub(crate) fn al_check(backend: &mut dyn AlErrorBackend) -> Result<(), AlError> {
    let error = backend.get_error();
    let text = match error {
        0 => return Ok(()),
        40961 => "AL_INVALID_NAME",
        40962 => "AL_INVALID_ENUM",
        40963 => "AL_INVALID_VALUE",
        40964 => "AL_INVALID_OPERATION",
        40965 => "AL_OUT_OF_MEMORY",
        _ => return Err(AlError(format!("Unknown AL Error: {error}"))),
    };
    Err(AlError(text.into()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consumes_one_error_and_preserves_all_exception_messages() {
        struct Backend(i32, usize);
        impl AlErrorBackend for Backend {
            fn get_error(&mut self) -> i32 {
                self.1 += 1;
                self.0
            }
        }
        for (error, text) in [
            (0, None),
            (40961, Some("AL_INVALID_NAME")),
            (40962, Some("AL_INVALID_ENUM")),
            (40963, Some("AL_INVALID_VALUE")),
            (40964, Some("AL_INVALID_OPERATION")),
            (40965, Some("AL_OUT_OF_MEMORY")),
            (-1, Some("Unknown AL Error: -1")),
        ] {
            let mut backend = Backend(error, 0);
            assert_eq!(
                al_check(&mut backend).err().map(|e| e.0),
                text.map(str::to_owned)
            );
            assert_eq!(backend.1, 1);
        }
    }
}
