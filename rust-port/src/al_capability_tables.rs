//! Generated from the supplied LWJGL 3.2.3 classes; see capability evidence.
use super::al_capabilities::{Flag, Symbol};
pub(crate) const ALC_SYMBOLS: &[Symbol] = &[
    Symbol {
        name: "alcOpenDevice",
        device: false,
    },
    Symbol {
        name: "alcCloseDevice",
        device: false,
    },
    Symbol {
        name: "alcCreateContext",
        device: false,
    },
    Symbol {
        name: "alcMakeContextCurrent",
        device: false,
    },
    Symbol {
        name: "alcProcessContext",
        device: false,
    },
    Symbol {
        name: "alcSuspendContext",
        device: false,
    },
    Symbol {
        name: "alcDestroyContext",
        device: false,
    },
    Symbol {
        name: "alcGetCurrentContext",
        device: false,
    },
    Symbol {
        name: "alcGetContextsDevice",
        device: false,
    },
    Symbol {
        name: "alcIsExtensionPresent",
        device: false,
    },
    Symbol {
        name: "alcGetProcAddress",
        device: false,
    },
    Symbol {
        name: "alcGetEnumValue",
        device: false,
    },
    Symbol {
        name: "alcGetError",
        device: false,
    },
    Symbol {
        name: "alcGetString",
        device: false,
    },
    Symbol {
        name: "alcGetIntegerv",
        device: false,
    },
    Symbol {
        name: "alcCaptureOpenDevice",
        device: false,
    },
    Symbol {
        name: "alcCaptureCloseDevice",
        device: false,
    },
    Symbol {
        name: "alcCaptureStart",
        device: false,
    },
    Symbol {
        name: "alcCaptureStop",
        device: false,
    },
    Symbol {
        name: "alcCaptureSamples",
        device: false,
    },
    Symbol {
        name: "alcSetThreadContext",
        device: true,
    },
    Symbol {
        name: "alcGetThreadContext",
        device: true,
    },
    Symbol {
        name: "alcGetInteger64vSOFT",
        device: true,
    },
    Symbol {
        name: "alcGetStringiSOFT",
        device: true,
    },
    Symbol {
        name: "alcResetDeviceSOFT",
        device: true,
    },
    Symbol {
        name: "alcLoopbackOpenDeviceSOFT",
        device: true,
    },
    Symbol {
        name: "alcIsRenderFormatSupportedSOFT",
        device: true,
    },
    Symbol {
        name: "alcRenderSamplesSOFT",
        device: true,
    },
    Symbol {
        name: "alcDevicePauseSOFT",
        device: true,
    },
    Symbol {
        name: "alcDeviceResumeSOFT",
        device: true,
    },
];
pub(crate) const ALC_FLAGS: &[Flag] = &[
    Flag {
        name: "OpenALC10",
        required: &[
            "alcOpenDevice",
            "alcCloseDevice",
            "alcCreateContext",
            "alcMakeContextCurrent",
            "alcProcessContext",
            "alcSuspendContext",
            "alcDestroyContext",
            "alcGetCurrentContext",
            "alcGetContextsDevice",
            "alcIsExtensionPresent",
            "alcGetProcAddress",
            "alcGetEnumValue",
            "alcGetError",
            "alcGetString",
            "alcGetIntegerv",
        ],
    },
    Flag {
        name: "OpenALC11",
        required: &[
            "alcCaptureOpenDevice",
            "alcCaptureCloseDevice",
            "alcCaptureStart",
            "alcCaptureStop",
            "alcCaptureSamples",
        ],
    },
    Flag {
        name: "ALC_ENUMERATE_ALL_EXT",
        required: &[],
    },
    Flag {
        name: "ALC_ENUMERATION_EXT",
        required: &[],
    },
    Flag {
        name: "ALC_EXT_CAPTURE",
        required: &[
            "alcCaptureOpenDevice",
            "alcCaptureCloseDevice",
            "alcCaptureStart",
            "alcCaptureStop",
            "alcCaptureSamples",
        ],
    },
    Flag {
        name: "ALC_EXT_DEDICATED",
        required: &[],
    },
    Flag {
        name: "ALC_EXT_DEFAULT_FILTER_ORDER",
        required: &[],
    },
    Flag {
        name: "ALC_EXT_disconnect",
        required: &[],
    },
    Flag {
        name: "ALC_EXT_EFX",
        required: &[],
    },
    Flag {
        name: "ALC_EXT_thread_local_context",
        required: &["alcSetThreadContext", "alcGetThreadContext"],
    },
    Flag {
        name: "ALC_LOKI_audio_channel",
        required: &[],
    },
    Flag {
        name: "ALC_SOFT_device_clock",
        required: &["alcGetInteger64vSOFT"],
    },
    Flag {
        name: "ALC_SOFT_HRTF",
        required: &["alcGetStringiSOFT", "alcResetDeviceSOFT"],
    },
    Flag {
        name: "ALC_SOFT_loopback",
        required: &[
            "alcLoopbackOpenDeviceSOFT",
            "alcIsRenderFormatSupportedSOFT",
            "alcRenderSamplesSOFT",
        ],
    },
    Flag {
        name: "ALC_SOFT_output_limiter",
        required: &[],
    },
    Flag {
        name: "ALC_SOFT_pause_device",
        required: &["alcDevicePauseSOFT", "alcDeviceResumeSOFT"],
    },
];
pub(crate) const AL_SYMBOLS: &[Symbol] = &[
    Symbol {
        name: "alGetError",
        device: false,
    },
    Symbol {
        name: "alEnable",
        device: false,
    },
    Symbol {
        name: "alDisable",
        device: false,
    },
    Symbol {
        name: "alIsEnabled",
        device: false,
    },
    Symbol {
        name: "alGetBoolean",
        device: false,
    },
    Symbol {
        name: "alGetInteger",
        device: false,
    },
    Symbol {
        name: "alGetFloat",
        device: false,
    },
    Symbol {
        name: "alGetDouble",
        device: false,
    },
    Symbol {
        name: "alGetBooleanv",
        device: false,
    },
    Symbol {
        name: "alGetIntegerv",
        device: false,
    },
    Symbol {
        name: "alGetFloatv",
        device: false,
    },
    Symbol {
        name: "alGetDoublev",
        device: false,
    },
    Symbol {
        name: "alGetString",
        device: false,
    },
    Symbol {
        name: "alDistanceModel",
        device: false,
    },
    Symbol {
        name: "alDopplerFactor",
        device: false,
    },
    Symbol {
        name: "alDopplerVelocity",
        device: false,
    },
    Symbol {
        name: "alListenerf",
        device: false,
    },
    Symbol {
        name: "alListeneri",
        device: false,
    },
    Symbol {
        name: "alListener3f",
        device: false,
    },
    Symbol {
        name: "alListenerfv",
        device: false,
    },
    Symbol {
        name: "alGetListenerf",
        device: false,
    },
    Symbol {
        name: "alGetListeneri",
        device: false,
    },
    Symbol {
        name: "alGetListener3f",
        device: false,
    },
    Symbol {
        name: "alGetListenerfv",
        device: false,
    },
    Symbol {
        name: "alGenSources",
        device: false,
    },
    Symbol {
        name: "alDeleteSources",
        device: false,
    },
    Symbol {
        name: "alIsSource",
        device: false,
    },
    Symbol {
        name: "alSourcef",
        device: false,
    },
    Symbol {
        name: "alSource3f",
        device: false,
    },
    Symbol {
        name: "alSourcefv",
        device: false,
    },
    Symbol {
        name: "alSourcei",
        device: false,
    },
    Symbol {
        name: "alGetSourcef",
        device: false,
    },
    Symbol {
        name: "alGetSource3f",
        device: false,
    },
    Symbol {
        name: "alGetSourcefv",
        device: false,
    },
    Symbol {
        name: "alGetSourcei",
        device: false,
    },
    Symbol {
        name: "alGetSourceiv",
        device: false,
    },
    Symbol {
        name: "alSourceQueueBuffers",
        device: false,
    },
    Symbol {
        name: "alSourceUnqueueBuffers",
        device: false,
    },
    Symbol {
        name: "alSourcePlay",
        device: false,
    },
    Symbol {
        name: "alSourcePause",
        device: false,
    },
    Symbol {
        name: "alSourceStop",
        device: false,
    },
    Symbol {
        name: "alSourceRewind",
        device: false,
    },
    Symbol {
        name: "alSourcePlayv",
        device: false,
    },
    Symbol {
        name: "alSourcePausev",
        device: false,
    },
    Symbol {
        name: "alSourceStopv",
        device: false,
    },
    Symbol {
        name: "alSourceRewindv",
        device: false,
    },
    Symbol {
        name: "alGenBuffers",
        device: false,
    },
    Symbol {
        name: "alDeleteBuffers",
        device: false,
    },
    Symbol {
        name: "alIsBuffer",
        device: false,
    },
    Symbol {
        name: "alGetBufferf",
        device: false,
    },
    Symbol {
        name: "alGetBufferi",
        device: false,
    },
    Symbol {
        name: "alBufferData",
        device: false,
    },
    Symbol {
        name: "alGetEnumValue",
        device: false,
    },
    Symbol {
        name: "alGetProcAddress",
        device: false,
    },
    Symbol {
        name: "alIsExtensionPresent",
        device: false,
    },
    Symbol {
        name: "alListener3i",
        device: false,
    },
    Symbol {
        name: "alGetListeneriv",
        device: false,
    },
    Symbol {
        name: "alSource3i",
        device: false,
    },
    Symbol {
        name: "alListeneriv",
        device: false,
    },
    Symbol {
        name: "alSourceiv",
        device: false,
    },
    Symbol {
        name: "alBufferf",
        device: false,
    },
    Symbol {
        name: "alBuffer3f",
        device: false,
    },
    Symbol {
        name: "alBufferfv",
        device: false,
    },
    Symbol {
        name: "alBufferi",
        device: false,
    },
    Symbol {
        name: "alBuffer3i",
        device: false,
    },
    Symbol {
        name: "alBufferiv",
        device: false,
    },
    Symbol {
        name: "alGetBufferiv",
        device: false,
    },
    Symbol {
        name: "alGetBufferfv",
        device: false,
    },
    Symbol {
        name: "alSpeedOfSound",
        device: false,
    },
    Symbol {
        name: "alGenEffects",
        device: false,
    },
    Symbol {
        name: "alDeleteEffects",
        device: false,
    },
    Symbol {
        name: "alIsEffect",
        device: false,
    },
    Symbol {
        name: "alEffecti",
        device: false,
    },
    Symbol {
        name: "alEffectiv",
        device: false,
    },
    Symbol {
        name: "alEffectf",
        device: false,
    },
    Symbol {
        name: "alEffectfv",
        device: false,
    },
    Symbol {
        name: "alGetEffecti",
        device: false,
    },
    Symbol {
        name: "alGetEffectiv",
        device: false,
    },
    Symbol {
        name: "alGetEffectf",
        device: false,
    },
    Symbol {
        name: "alGetEffectfv",
        device: false,
    },
    Symbol {
        name: "alGenFilters",
        device: false,
    },
    Symbol {
        name: "alDeleteFilters",
        device: false,
    },
    Symbol {
        name: "alIsFilter",
        device: false,
    },
    Symbol {
        name: "alFilteri",
        device: false,
    },
    Symbol {
        name: "alFilteriv",
        device: false,
    },
    Symbol {
        name: "alFilterf",
        device: false,
    },
    Symbol {
        name: "alFilterfv",
        device: false,
    },
    Symbol {
        name: "alGetFilteri",
        device: false,
    },
    Symbol {
        name: "alGetFilteriv",
        device: false,
    },
    Symbol {
        name: "alGetFilterf",
        device: false,
    },
    Symbol {
        name: "alGetFilterfv",
        device: false,
    },
    Symbol {
        name: "alGenAuxiliaryEffectSlots",
        device: false,
    },
    Symbol {
        name: "alDeleteAuxiliaryEffectSlots",
        device: false,
    },
    Symbol {
        name: "alIsAuxiliaryEffectSlot",
        device: false,
    },
    Symbol {
        name: "alAuxiliaryEffectSloti",
        device: false,
    },
    Symbol {
        name: "alAuxiliaryEffectSlotiv",
        device: false,
    },
    Symbol {
        name: "alAuxiliaryEffectSlotf",
        device: false,
    },
    Symbol {
        name: "alAuxiliaryEffectSlotfv",
        device: false,
    },
    Symbol {
        name: "alGetAuxiliaryEffectSloti",
        device: false,
    },
    Symbol {
        name: "alGetAuxiliaryEffectSlotiv",
        device: false,
    },
    Symbol {
        name: "alGetAuxiliaryEffectSlotf",
        device: false,
    },
    Symbol {
        name: "alGetAuxiliaryEffectSlotfv",
        device: false,
    },
    Symbol {
        name: "alBufferDataStatic",
        device: false,
    },
    Symbol {
        name: "alDeferUpdatesSOFT",
        device: false,
    },
    Symbol {
        name: "alProcessUpdatesSOFT",
        device: false,
    },
    Symbol {
        name: "alSourcedSOFT",
        device: false,
    },
    Symbol {
        name: "alSource3dSOFT",
        device: false,
    },
    Symbol {
        name: "alSourcedvSOFT",
        device: false,
    },
    Symbol {
        name: "alGetSourcedSOFT",
        device: false,
    },
    Symbol {
        name: "alGetSource3dSOFT",
        device: false,
    },
    Symbol {
        name: "alGetSourcedvSOFT",
        device: false,
    },
    Symbol {
        name: "alSourcei64SOFT",
        device: false,
    },
    Symbol {
        name: "alSource3i64SOFT",
        device: false,
    },
    Symbol {
        name: "alSourcei64vSOFT",
        device: false,
    },
    Symbol {
        name: "alGetSourcei64SOFT",
        device: false,
    },
    Symbol {
        name: "alGetSource3i64SOFT",
        device: false,
    },
    Symbol {
        name: "alGetSourcei64vSOFT",
        device: false,
    },
    Symbol {
        name: "alGetStringiSOFT",
        device: false,
    },
];
pub(crate) const AL_FLAGS: &[Flag] = &[
    Flag {
        name: "OpenAL10",
        required: &[
            "alGetError",
            "alEnable",
            "alDisable",
            "alIsEnabled",
            "alGetBoolean",
            "alGetInteger",
            "alGetFloat",
            "alGetDouble",
            "alGetBooleanv",
            "alGetIntegerv",
            "alGetFloatv",
            "alGetDoublev",
            "alGetString",
            "alDistanceModel",
            "alDopplerFactor",
            "alDopplerVelocity",
            "alListenerf",
            "alListeneri",
            "alListener3f",
            "alListenerfv",
            "alGetListenerf",
            "alGetListeneri",
            "alGetListener3f",
            "alGetListenerfv",
            "alGenSources",
            "alDeleteSources",
            "alIsSource",
            "alSourcef",
            "alSource3f",
            "alSourcefv",
            "alSourcei",
            "alGetSourcef",
            "alGetSource3f",
            "alGetSourcefv",
            "alGetSourcei",
            "alGetSourceiv",
            "alSourceQueueBuffers",
            "alSourceUnqueueBuffers",
            "alSourcePlay",
            "alSourcePause",
            "alSourceStop",
            "alSourceRewind",
            "alSourcePlayv",
            "alSourcePausev",
            "alSourceStopv",
            "alSourceRewindv",
            "alGenBuffers",
            "alDeleteBuffers",
            "alIsBuffer",
            "alGetBufferf",
            "alGetBufferi",
            "alBufferData",
            "alGetEnumValue",
            "alGetProcAddress",
            "alIsExtensionPresent",
        ],
    },
    Flag {
        name: "OpenAL11",
        required: &[
            "alListener3i",
            "alGetListeneriv",
            "alSource3i",
            "alListeneriv",
            "alSourceiv",
            "alBufferf",
            "alBuffer3f",
            "alBufferfv",
            "alBufferi",
            "alBuffer3i",
            "alBufferiv",
            "alGetBufferiv",
            "alGetBufferfv",
            "alSpeedOfSound",
        ],
    },
    Flag {
        name: "AL_EXT_ALAW",
        required: &[],
    },
    Flag {
        name: "AL_EXT_BFORMAT",
        required: &[],
    },
    Flag {
        name: "AL_EXT_DOUBLE",
        required: &[],
    },
    Flag {
        name: "ALC_EXT_EFX",
        required: &[
            "alGenEffects",
            "alDeleteEffects",
            "alIsEffect",
            "alEffecti",
            "alEffectiv",
            "alEffectf",
            "alEffectfv",
            "alGetEffecti",
            "alGetEffectiv",
            "alGetEffectf",
            "alGetEffectfv",
            "alGenFilters",
            "alDeleteFilters",
            "alIsFilter",
            "alFilteri",
            "alFilteriv",
            "alFilterf",
            "alFilterfv",
            "alGetFilteri",
            "alGetFilteriv",
            "alGetFilterf",
            "alGetFilterfv",
            "alGenAuxiliaryEffectSlots",
            "alDeleteAuxiliaryEffectSlots",
            "alIsAuxiliaryEffectSlot",
            "alAuxiliaryEffectSloti",
            "alAuxiliaryEffectSlotiv",
            "alAuxiliaryEffectSlotf",
            "alAuxiliaryEffectSlotfv",
            "alGetAuxiliaryEffectSloti",
            "alGetAuxiliaryEffectSlotiv",
            "alGetAuxiliaryEffectSlotf",
            "alGetAuxiliaryEffectSlotfv",
        ],
    },
    Flag {
        name: "AL_EXT_EXPONENT_DISTANCE",
        required: &[],
    },
    Flag {
        name: "AL_EXT_FLOAT32",
        required: &[],
    },
    Flag {
        name: "AL_EXT_IMA4",
        required: &[],
    },
    Flag {
        name: "AL_EXT_LINEAR_DISTANCE",
        required: &[],
    },
    Flag {
        name: "AL_EXT_MCFORMATS",
        required: &[],
    },
    Flag {
        name: "AL_EXT_MULAW",
        required: &[],
    },
    Flag {
        name: "AL_EXT_MULAW_BFORMAT",
        required: &[],
    },
    Flag {
        name: "AL_EXT_MULAW_MCFORMATS",
        required: &[],
    },
    Flag {
        name: "AL_EXT_OFFSET",
        required: &[],
    },
    Flag {
        name: "AL_EXT_source_distance_model",
        required: &[],
    },
    Flag {
        name: "AL_EXT_SOURCE_RADIUS",
        required: &[],
    },
    Flag {
        name: "AL_EXT_static_buffer",
        required: &["alBufferDataStatic"],
    },
    Flag {
        name: "AL_EXT_STEREO_ANGLES",
        required: &[],
    },
    Flag {
        name: "AL_EXT_vorbis",
        required: &[],
    },
    Flag {
        name: "AL_LOKI_IMA_ADPCM",
        required: &[],
    },
    Flag {
        name: "AL_LOKI_quadriphonic",
        required: &[],
    },
    Flag {
        name: "AL_LOKI_WAVE_format",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_block_alignment",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_deferred_updates",
        required: &["alDeferUpdatesSOFT", "alProcessUpdatesSOFT"],
    },
    Flag {
        name: "AL_SOFT_direct_channels",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_gain_clamp_ex",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_loop_points",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_MSADPCM",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_source_latency",
        required: &[
            "alSourcedSOFT",
            "alSource3dSOFT",
            "alSourcedvSOFT",
            "alGetSourcedSOFT",
            "alGetSource3dSOFT",
            "alGetSourcedvSOFT",
            "alSourcei64SOFT",
            "alSource3i64SOFT",
            "alSourcei64vSOFT",
            "alGetSourcei64SOFT",
            "alGetSource3i64SOFT",
            "alGetSourcei64vSOFT",
        ],
    },
    Flag {
        name: "AL_SOFT_source_length",
        required: &[],
    },
    Flag {
        name: "AL_SOFT_source_resampler",
        required: &["alGetStringiSOFT"],
    },
    Flag {
        name: "AL_SOFT_source_spatialize",
        required: &[],
    },
];
