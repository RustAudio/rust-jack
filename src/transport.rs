///! JACK transport wrappers.
///! See the [transport design api docs](https://jackaudio.org/api/transport-design.html) for more info.
use crate::{Frames, Time};
use jack_sys as j;
use std::sync::Weak;

type Result<T> = ::std::result::Result<T, crate::Error>;

/// A structure for querying and manipulating the JACK transport.
pub struct Transport {
    pub(crate) client_ptr: *mut j::jack_client_t,
    pub(crate) client_life: Weak<()>,
}

/// A structure representing the transport position.
#[repr(transparent)]
pub struct TransportPosition(j::jack_position_t);

/// A representation of transport state.
#[derive(Debug, Clone, Copy)]
pub enum TransportState {
    Stopped,
    Rolling,
    Starting,
}

/// A helper struct encapsulating both `TransportState` and `TransportPosition`.
pub struct TransportStatePosition {
    pub pos: TransportPosition,
    pub state: TransportState,
}

/// Transport Bar Beat Tick data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportBBT {
    /// Time signature bar, 1 or more.
    pub bar: usize,
    /// Time signature beat, 1 <= beat <= sig_num.
    pub beat: usize,
    /// current tick-within-beat
    /// # Remarks
    /// * Should be >= 0 and < ticks_per_beat: the first tick is tick 0.
    pub tick: usize,
    /// Time Signature "numerator". Jack calls this `beats_per_bar`.
    pub sig_num: f32,
    /// Time Signature "denominator". Jack calls this `beat_type`.
    pub sig_denom: f32,
    /// Number of ticks within a beat.
    /// # Remarks
    /// * Usually a moderately large integer with many denominators, such as 1920.0
    pub ticks_per_beat: f64,
    /// BPM, quantized to block size. This means when the tempo is not constant within this block,
    /// the BPM value should adapted to compensate for this. This is different from most fields in
    /// this struct, which specify the value at the beginning of the block rather than an average.
    pub bpm: f64,
    /// Number of ticks that have elapsed between frame 0 and the first beat of the current measure.
    pub bar_start_tick: f64,
}

impl Transport {
    fn with_client<F: Fn(*mut j::jack_client_t) -> R, R>(&self, func: F) -> Result<R> {
        if let Some(_) = self.client_life.upgrade() {
            Ok(func(self.client_ptr))
        } else {
            Err(crate::Error::ClientIsNoLongerAlive)
        }
    }

    /// Start the JACK transport rolling.
    ///
    /// # Remarks
    ///
    /// * Any client can make this request at any time.
    /// * It takes effect no sooner than the next process cycle, perhaps later if there are
    /// slow-sync clients.
    /// * This function is realtime-safe.
    pub fn start(&self) -> Result<()> {
        self.with_client(|ptr| unsafe {
            j::jack_transport_start(ptr);
        })
    }

    /// Stop the JACK transport.
    ///
    /// # Remarks
    ///
    /// * Any client can make this request at any time.
    /// * It takes effect on the next process cycle.
    /// * This function is realtime-safe.
    pub fn stop(&self) -> Result<()> {
        self.with_client(|ptr| unsafe {
            j::jack_transport_stop(ptr);
        })
    }

    /// Request a new transport position.
    ///
    /// # Arguments
    ///
    /// * `pos` - requested new transport position.
    ///
    /// # Remarks
    ///
    /// * May be called at any time by any client.
    /// * The new position takes effect in two process cycles.
    /// * If there are slow-sync clients and the transport is already rolling, it will enter the `TransportState::Starting` state and begin invoking their sync_callbacks until ready.
    /// * This function is realtime-safe.
    pub fn reposition(&self, pos: &TransportPosition) -> Result<()> {
        Self::to_result(
            self.with_client(|ptr| unsafe {
                j::jack_transport_reposition(
                    ptr,
                    std::mem::transmute::<&TransportPosition, *const j::jack_position_t>(pos),
                )
            }),
            (),
        )
    }

    /// Reposition the transport to a new frame number.
    ///
    /// # Arguments
    ///
    /// * `frame` - frame number of new transport position.
    ///
    /// # Remarks
    ///
    /// * May be called at any time by any client.
    /// * The new position takes effect in two process cycles.
    /// * If there are slow-sync clients and the transport is already rolling, it will enter the JackTransportStarting state and begin invoking their sync_callbacks until ready.
    /// * This function is realtime-safe.
    pub fn locate(&self, frame: Frames) -> Result<()> {
        Self::to_result(
            self.with_client(|ptr| unsafe { j::jack_transport_locate(ptr, frame) }),
            (),
        )
    }

    //helper to convert to TransportState
    fn to_state(state: j::jack_transport_state_t) -> TransportState {
        match state {
            j::JackTransportStopped => TransportState::Stopped,
            j::JackTransportStarting => TransportState::Starting,
            _ => TransportState::Rolling,
        }
    }

    //helper to create generic error from jack response
    fn to_result<R>(v: Result<::libc::c_int>, r: R) -> Result<R> {
        match v {
            Ok(v) => {
                if v == 0 {
                    Ok(r)
                } else {
                    Err(crate::Error::UnknownError)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Query the current transport state and position.
    ///
    /// # Remarks
    ///
    /// * This function is realtime-safe, and can be called from any thread.
    /// * If called from the process thread, `pos` corresponds to the first frame of the current cycle and the state returned is valid for the entire cycle.
    pub fn query(&self) -> Result<TransportStatePosition> {
        self.with_client(|ptr| {
            let mut pos: std::mem::MaybeUninit<TransportPosition> = std::mem::MaybeUninit::zeroed();
            let state = Self::to_state(unsafe {
                j::jack_transport_query(
                    ptr,
                    std::mem::transmute::<*mut TransportPosition, *mut j::jack_position_t>(
                        pos.as_mut_ptr(),
                    ),
                )
            });
            TransportStatePosition {
                pos: unsafe { pos.assume_init() },
                state,
            }
        })
    }

    /// Query the current transport state.
    ///
    /// # Remarks
    ///
    /// * This function is realtime-safe, and can be called from any thread.
    /// * If called from the process thread, the state returned is valid for the entire cycle.
    pub fn query_state(&self) -> Result<TransportState> {
        self.with_client(|ptr| {
            Self::to_state(unsafe { j::jack_transport_query(ptr, std::ptr::null_mut()) })
        })
    }
}

impl TransportPosition {
    /// Query to see if the BarBeatsTick data is valid.
    pub fn valid_bbt(&self) -> bool {
        (self.0.valid & j::JackPositionBBT) != 0
    }

    /// Query to see if the frame offset of BarBeatsTick data is valid.
    pub fn valid_bbt_frame_offset(&self) -> bool {
        (self.0.valid & j::JackBBTFrameOffset) != 0
    }

    /*
    /// Query to see if the Timecode data is valid.
    pub fn valid_timecode(&self) -> bool {
        (self.0.valid & j::JackPositionTimecode) != 0
    }

    /// Query to see if the Audio/Video ratio is valid.
    pub fn valid_avr(&self) -> bool {
        (self.0.valid & j::JackAudioVideoRatio) != 0
    }

    /// Query to see if the Video frame offset is valid.
    pub fn valid_video_frame_offset(&self) -> bool {
        (self.0.valid & j::JackVideoFrameOffset) != 0
    }
    */

    /// Get the frame number on the transport timeline.
    ///
    /// # Remarks
    /// * This is not the same as what jack_frame_time returns.
    pub fn frame(&self) -> Frames {
        self.0.frame
    }

    /// Set the frame number on the transport timeline.
    pub fn set_frame(&mut self, frame: Frames) {
        self.0.frame = frame;
    }

    /// Get the current frame rate, in frames per second.
    ///
    /// # Remarks
    /// * This is only set by the server so it will be `None` if this struct hasn't come from the
    /// sever.
    pub fn frame_rate(&self) -> Option<Frames> {
        if self.0.frame_rate > 0 {
            Some(self.0.frame_rate)
        } else {
            None
        }
    }

    /// Get a microsecond timestamp.
    ///
    /// # Remarks
    /// * This is only set by the server so it will be `None` if this struct hasn't come from the
    /// sever.
    /// * Guaranteed to be monotonic, but not neccessarily linear.
    /// * The absolute value is implementation-dependent (i.e. it could be wall-clock, time since
    /// jack started, uptime, etc).
    pub fn usecs(&self) -> Option<Time> {
        //NOTE could it actually be 0 and come from the server?
        if self.0.usecs > 0 {
            Some(self.0.usecs)
        } else {
            None
        }
    }

    /// Get the BarBeatsTick data if it is valid.
    pub fn bbt(&self) -> Option<TransportBBT> {
        if self.valid_bbt() && self.0.bar > 0 && self.0.beat > 0 && self.0.tick >= 0 {
            Some(TransportBBT {
                bar: self.0.bar as _,
                beat: self.0.bar as _,
                tick: self.0.tick as _,
                sig_num: self.0.beats_per_bar,
                sig_denom: self.0.beat_type,
                ticks_per_beat: self.0.ticks_per_beat,
                bpm: self.0.beats_per_minute,
                bar_start_tick: self.0.bar_start_tick,
            })
        } else {
            None
        }
    }

    /// Set the BarBeatsTick data in this position.
    ///
    /// # Arguments
    /// * `bbt` - The data to set in the position. `None` will invalidate the BarBeatsTick data.
    ///
    /// # Remarks
    /// * On failure this will leave the pre-existing data intact.
    pub fn set_bbt(&mut self, bbt: Option<TransportBBT>) -> std::result::Result<(), TransportBBT> {
        match bbt {
            None => {
                self.0.valid = self.0.valid & !j::JackPositionBBT;
                Ok(())
            }
            Some(bbt) => {
                if bbt.bar < 1
                    || bbt.beat < 1
                    || bbt.sig_num <= 0.
                    || bbt.sig_denom <= 0.
                    || bbt.ticks_per_beat <= 0.
                    || bbt.bpm < 0.
                    || bbt.beat > bbt.sig_num.ceil() as _
                    || bbt.tick >= bbt.ticks_per_beat.ceil() as _
                {
                    Err(bbt)
                } else {
                    self.0.bar = bbt.bar as _;
                    self.0.beat = bbt.beat as _;
                    self.0.tick = bbt.tick as _;
                    self.0.beats_per_bar = bbt.sig_num;
                    self.0.beat_type = bbt.sig_denom;
                    self.0.ticks_per_beat = bbt.ticks_per_beat;
                    self.0.beats_per_minute = bbt.bpm;
                    self.0.bar_start_tick = bbt.bar_start_tick;
                    self.0.valid |= j::JackPositionBBT;
                    Ok(())
                }
            }
        }
    }

    /// Get the frame offset for the BBT fields.
    ///
    /// # Remarks
    ///
    /// * Should be assumed to be 0 if `None`.
    /// * If this value is Some(0), the bbt time refers to the first frame of this cycle.
    /// * Otherwise, the bbt time refers to a frame that many frames **before** the start of the cycle.
    pub fn bbt_offset(&self) -> Option<Frames> {
        if self.valid_bbt_frame_offset() {
            Some(self.0.bbt_offset)
        } else {
            None
        }
    }

    /// Set the frame offset for the BBT fields.
    ///
    /// # Arguments
    /// * `frame` - The value to set to the offset. `None` will invalidate the offset data.
    ///
    /// # Remarks
    ///
    /// * If this value is 0, the bbt time refers to the first frame of this cycle.
    /// * Otherwise, the bbt time refers to a frame that many frames **before** the start of the cycle.
    pub fn set_bbt_offset(&mut self, frame: Option<Frames>) -> std::result::Result<(), Frames> {
        match frame {
            None => {
                self.0.valid = self.0.valid & !j::JackBBTFrameOffset;
                Ok(())
            }
            Some(frame) => {
                self.0.bbt_offset = frame;
                self.0.valid |= j::JackBBTFrameOffset;
                Ok(())
            }
        }
    }
}

impl Default for TransportPosition {
    fn default() -> Self {
        //safe to zero
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
    }
}

impl Default for TransportBBT {
    fn default() -> Self {
        Self {
            bar: 1,
            beat: 1,
            tick: 0,
            sig_num: 4.,
            sig_denom: 4.,
            ticks_per_beat: 1920.,
            bpm: 120.,
            bar_start_tick: 0.,
        }
    }
}

#[cfg(test)]
mod test {
    mod position {
        use crate::{TransportBBT, TransportPosition};
        #[test]
        fn default() {
            let p: TransportPosition = Default::default();
            assert!(!p.valid_bbt());
            assert!(!p.valid_bbt_frame_offset());
            assert_eq!(p.frame(), 0);
            assert_eq!(p.bbt(), None);
            assert_eq!(p.bbt_offset(), None);
            assert_eq!(p.frame_rate(), None);
            assert_eq!(p.usecs(), None);
        }

        #[test]
        fn usecs() {
            let mut p: TransportPosition = Default::default();
            assert_eq!(p.usecs(), None);
            p.0.usecs = 1;
            assert_eq!(p.usecs(), Some(1));
            p.0.usecs = 0;
            assert_eq!(p.usecs(), None);
            p.0.usecs = 2084;
            assert_eq!(p.usecs(), Some(2084));
        }

        #[test]
        fn frame_rate() {
            let mut p: TransportPosition = Default::default();
            assert_eq!(p.frame_rate(), None);
            p.0.frame_rate = 44100;
            assert_eq!(p.frame_rate(), Some(44100));
            p.0.frame_rate = 0;
            assert_eq!(p.frame_rate(), None);
            p.0.frame_rate = 48000;
            assert_eq!(p.frame_rate(), Some(48000));
        }

        #[test]
        fn bbt_invalid() {
            let mut i: TransportPosition = Default::default();
            let mut v: TransportPosition = Default::default();
            let def: TransportBBT = Default::default();

            assert!(!i.valid_bbt());
            assert_eq!(i.set_bbt(None), Ok(()));
            assert!(!i.valid_bbt());

            assert!(!v.valid_bbt());
            assert_eq!(v.set_bbt(Some(def)), Ok(()));
            assert_eq!(v.bbt(), Some(def));

            let mut t = |b| {
                assert!(i.set_bbt(Some(b)).is_err());
                assert!(v.set_bbt(Some(b)).is_err());
                assert!(!i.valid_bbt());
                assert!(v.valid_bbt());
                assert_eq!(i.bbt(), None);
                assert_eq!(v.bbt(), Some(def));
            };

            let mut bbt: TransportBBT = Default::default();
            bbt.bar = 0;
            t(bbt);

            bbt = Default::default();
            bbt.beat = 0;
            t(bbt);

            bbt.beat = 5;
            t(bbt);

            bbt = Default::default();
            bbt.tick = 1921;
            t(bbt);
            bbt.tick = 1920;
            t(bbt);

            bbt = Default::default();
            bbt.bpm = -1.0;
            t(bbt);

            bbt = Default::default();
            bbt.ticks_per_beat = -1.0;
            t(bbt);
            bbt.ticks_per_beat = 0.0;
            t(bbt);

            bbt = Default::default();
            bbt.sig_num = 0.0;
            t(bbt);
            bbt.sig_num = -1.0;
            t(bbt);

            bbt = Default::default();
            bbt.sig_denom = 0.0;
            t(bbt);
            bbt.sig_denom = -1.0;
            t(bbt);
        }
    }
    mod bbt {
        #[test]
        fn default() {
            let bbt: crate::TransportBBT = Default::default();
            assert_eq!(bbt.bar, 1);
            assert_eq!(bbt.beat, 1);
            assert_eq!(bbt.tick, 0);
            assert_eq!(bbt.sig_num, 4.0);
            assert_eq!(bbt.sig_denom, 4.0);
            assert_eq!(bbt.ticks_per_beat, 1920.0);
            assert_eq!(bbt.bpm, 120.0);
            assert_eq!(bbt.bar_start_tick, 0.0);
        }
    }
}
