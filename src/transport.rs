//! JACK transport wrappers.
//! See the [transport design api docs](https://jackaudio.org/api/transport-design.html) for more info.
use crate::{Frames, Time};
use jack_sys as j;
use std::sync::Weak;

pub type Result<T> = ::std::result::Result<T, crate::Error>;

/// A structure for querying and manipulating the JACK transport.
pub struct Transport {
    pub(crate) client_ptr: *mut j::jack_client_t,
    pub(crate) client_life: Weak<()>,
}

//all exposed methods are realtime safe
unsafe impl Send for Transport {}
unsafe impl Sync for Transport {}

/// A structure representing the transport position.
#[repr(transparent)]
pub struct TransportPosition(j::jack_position_t);

/// A representation of transport state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    Stopped,
    Rolling,
    Starting,
}

/// A helper struct encapsulating both [`TransportState`] and [`TransportPosition`].
#[derive(Debug)]
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

/// An error validating a TransportBBT
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportBBTValidationError {
    ///`bar` must be greater than zero
    BarZero,
    ///`beat` must be greater than zero and less than `sig_num`
    BeatRange,
    ///There must more than zero ticks per beat
    TicksPerBeatRange,
    ///Time signature numerator, `sig_num` must be greater than zero
    SigNumRange,
    ///Time signature denominator, `sig_denom` must be greater than zero
    SigDenomRange,
    ///`bpm` must be greater than or equal to zero
    BPMRange,
    ///`tick` must be less than `ticks_per_beat`
    TickRange,
}

impl std::fmt::Display for TransportBBTValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TransportBBTValidationError: {:?}", &self)
    }
}

impl std::error::Error for TransportBBTValidationError {}

impl Transport {
    /// Start the JACK transport rolling.
    ///
    /// # Remarks
    ///
    /// * Any client can make this request at any time.
    /// * It takes effect no sooner than the next process cycle, perhaps later if there are
    ///   slow-sync clients.
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
    /// * If there are slow-sync clients and the transport is already rolling, it will enter the [`TransportState::Starting`] state and begin invoking their sync_callbacks until ready.
    /// * This function is realtime-safe.
    pub fn reposition(&self, pos: &TransportPosition) -> Result<()> {
        Self::result_from_ffi(
            self.with_client(|ptr| unsafe {
                j::jack_transport_reposition(ptr, &pos.0 as *const j::jack_position_t)
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
        Self::result_from_ffi(
            self.with_client(|ptr| unsafe { j::jack_transport_locate(ptr, frame) }),
            (),
        )
    }

    // Helper to convert to TransportState
    pub(crate) fn state_from_ffi(state: j::jack_transport_state_t) -> TransportState {
        match state {
            j::JackTransportStopped => TransportState::Stopped,
            j::JackTransportStarting => TransportState::Starting,
            //the JackTransportLooping state is no longer used
            _ => TransportState::Rolling,
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
            let state = Self::state_from_ffi(unsafe {
                j::jack_transport_query(
                    ptr,
                    pos.as_mut_ptr() as *mut jack_sys::Struct__jack_position,
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
            Self::state_from_ffi(unsafe { j::jack_transport_query(ptr, std::ptr::null_mut()) })
        })
    }

    fn with_client<F: Fn(*mut j::jack_client_t) -> R, R>(&self, func: F) -> Result<R> {
        if self.client_life.upgrade().is_some() {
            Ok(func(self.client_ptr))
        } else {
            Err(crate::Error::ClientIsNoLongerAlive)
        }
    }

    // Helper to create generic error from jack response
    fn result_from_ffi<R>(v: Result<::libc::c_int>, r: R) -> Result<R> {
        match v {
            Ok(0) => Ok(r),
            Ok(error_code) => Err(crate::Error::UnknownError { error_code }),
            Err(e) => Err(e),
        }
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
    ///   server.
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
    ///   server.
    /// * Guaranteed to be monotonic, but not necessarily linear.
    /// * The absolute value is implementation-dependent (i.e. it could be wall-clock, time since
    ///   jack started, uptime, etc).
    pub fn usecs(&self) -> Option<Time> {
        // NOTE could it actually be 0 and come from the server?
        if self.0.usecs > 0 {
            Some(self.0.usecs)
        } else {
            None
        }
    }

    /// Get the BarBeatsTick data if it is valid.
    pub fn bbt(&self) -> Option<TransportBBT> {
        if self.valid_bbt() {
            TransportBBT {
                bar: self.0.bar as _,
                beat: self.0.beat as _,
                tick: self.0.tick as _,
                sig_num: self.0.beats_per_bar,
                sig_denom: self.0.beat_type,
                ticks_per_beat: self.0.ticks_per_beat,
                bpm: self.0.beats_per_minute,
                bar_start_tick: self.0.bar_start_tick,
            }
            .validated()
            .ok()
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
    /// * If `bbt` is not valid, will leave the pre-existing data intact.
    pub fn set_bbt(
        &mut self,
        bbt: Option<TransportBBT>,
    ) -> std::result::Result<(), TransportBBTValidationError> {
        match bbt {
            None => {
                self.0.valid &= !j::JackPositionBBT;
                Ok(())
            }
            Some(bbt) => match bbt.validated() {
                Ok(bbt) => {
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
                Err(e) => Err(e),
            },
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
                self.0.valid &= !j::JackBBTFrameOffset;
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

impl std::fmt::Debug for TransportPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("TransportPosition");
        let unique_1 = self.0.unique_1;
        let usecs = self.0.usecs;
        let frame = self.0.frame;
        let frame_rate = self.0.frame_rate;
        let valid = self.0.valid != 0;
        let mut d = s
            .field("unique_1", &unique_1)
            .field("usecs", &usecs)
            .field("frame", &frame)
            .field("frame_rate", &frame_rate)
            .field("valid", &valid);
        if let Some(bbt) = self.bbt() {
            d = d.field("bbt", &bbt);
        }
        d.finish()
    }
}

impl TransportBBT {
    /// Set bar, beat, tick
    ///
    /// # Example
    /// ```
    /// use jack::TransportBBT;
    /// let bbt = TransportBBT::default().with_bbt(4, 2, 14).validated();
    /// assert!(bbt.is_ok());
    /// let bbt = bbt.unwrap();
    /// assert_eq!(bbt.bar, 4);
    /// assert_eq!(bbt.beat, 2);
    /// assert_eq!(bbt.tick, 14);
    /// ```
    pub fn with_bbt(&'_ mut self, bar: usize, beat: usize, tick: usize) -> &'_ mut Self {
        self.bar = bar;
        self.beat = beat;
        self.tick = tick;
        self
    }

    /// Set Beats Per Minute.
    pub fn with_bpm(&'_ mut self, bpm: f64) -> &'_ mut Self {
        self.bpm = bpm;
        self
    }

    /// Set the time signature.
    pub fn with_timesig(&'_ mut self, num: f32, denom: f32) -> &'_ mut Self {
        self.sig_num = num;
        self.sig_denom = denom;
        self
    }

    /// Set ticks per beat.
    pub fn with_ticks_per_beat(&'_ mut self, ticks_per_beat: f64) -> &'_ mut Self {
        self.ticks_per_beat = ticks_per_beat;
        self
    }

    /// Set bar start tick.
    pub fn with_bar_start_tick(&'_ mut self, tick: f64) -> &'_ mut Self {
        self.bar_start_tick = tick;
        self
    }

    /// Returns `self` is valid, otherwise returns an error describing what is invalid.
    pub fn validated(&'_ self) -> std::result::Result<Self, TransportBBTValidationError> {
        if self.bar == 0 {
            Err(TransportBBTValidationError::BarZero)
        } else if self.beat == 0 || self.beat > self.sig_num.ceil() as _ {
            Err(TransportBBTValidationError::BeatRange)
        } else if self.ticks_per_beat <= 0. {
            Err(TransportBBTValidationError::TicksPerBeatRange)
        } else if self.sig_num <= 0. {
            Err(TransportBBTValidationError::SigNumRange)
        } else if self.sig_denom <= 0. {
            Err(TransportBBTValidationError::SigDenomRange)
        } else if self.bpm < 0. {
            Err(TransportBBTValidationError::BPMRange)
        } else if self.tick >= self.ticks_per_beat.ceil() as _ {
            Err(TransportBBTValidationError::TickRange)
        } else {
            Ok(*self)
        }
    }

    /// Returns true if valid. Use `validated` to get the exact validation results.
    pub fn valid(&self) -> bool {
        self.validated().is_ok()
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
