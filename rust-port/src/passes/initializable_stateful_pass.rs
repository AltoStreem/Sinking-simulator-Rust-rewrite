//! InitializableStatefulPass.java combines setup and uniform forwarding.
use super::{initializable_pass::InitializablePass, stateful_pass::StatefulPass};
pub(crate) trait InitializableStatefulPass: InitializablePass + StatefulPass {}
