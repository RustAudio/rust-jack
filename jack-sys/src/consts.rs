/// JACK port type for 8 bit raw midi
pub static RAW_MIDI_TYPE: &str = "8 bit raw midi";

/// JACK port type for 32 bit float mono audio
pub static FLOAT_MONO_AUDIO: &str = "32 bit float mono audio";

pub const JACK_LIB: &'static str = if cfg!(windows) {
    if cfg!(target_arch = "x86") {
        "libjack.dll"
    } else {
        "libjack64.dll"
    }
} else if cfg!(target_vendor = "apple") {
    "libjack.0.dylib"
} else {
    "libjack.so.0"
};
