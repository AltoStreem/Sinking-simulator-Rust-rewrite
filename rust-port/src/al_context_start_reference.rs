//! ALContext$start$1.java mutable property reference.
#![allow(dead_code)]
use crate::al_context::{AlContext, CapabilityObject};
use std::sync::Arc;
pub(crate) struct AlcCapabilitiesReference {
    pub receiver: Arc<AlContext>,
}
impl AlcCapabilitiesReference {
    pub fn name(&self) -> &'static str {
        "alcCapabilities"
    }
    pub fn signature(&self) -> &'static str {
        "getAlcCapabilities()Lorg/lwjgl/openal/ALCCapabilities;"
    }
    pub fn owner(&self) -> &'static str {
        "com.wicpar.engine.audio.ALContext"
    }
    pub fn get(&self) -> CapabilityObject {
        self.receiver.alc_capabilities()
    }
    pub fn set(&self, value: Option<CapabilityObject>) {
        self.receiver
            .set_alc_capabilities(value.expect("alcCapabilities setter requires a non-null value"));
    }
}
