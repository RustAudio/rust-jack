///! JACK transport wrappers.
///! See the [transport design api docs](https://jackaudio.org/api/transport-design.html) for more info.
use crate::{Frames, Time};
use jack_sys as j;
use std::sync::Weak;

pub type Result<T> = ::std::result::Result<T, crate::Error>;

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
            //the JackTransportLooping state is no longer used
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

//all exposed methods are realtime safe
unsafe impl Send for Transport {}
unsafe impl Sync for Transport {}

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
                beat: self.0.beat as _,
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
                if bbt.valid() {
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
                } else {
                    Err(bbt)
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

impl TransportBBT {
    /// Set bar, beat, tick
    ///
    /// # Example
    /// ```
    /// use jack::TransportBBT;
    /// let bbt = TransportBBT::default().with_bbt(4, 2, 14).validated();
    /// assert!(bbt.is_ok());
    /// assert_eq!(bbt.unwrap().bar, 4);
    /// assert_eq!(bbt.unwrap().beat, 2);
    /// assert_eq!(bbt.unwrap().tick, 14);
    /// ```
    pub fn with_bbt<'a>(&'a mut self, bar: usize, beat: usize, tick: usize) -> &'a mut Self {
        self.bar = bar;
        self.beat = beat;
        self.tick = tick;
        self
    }

    /// Set Beats Per Minute.
    pub fn with_bpm<'a>(&'a mut self, bpm: f64) -> &'a mut Self {
        self.bpm = bpm;
        self
    }

    /// Set the time signature.
    pub fn with_timesig<'a>(&'a mut self, num: f32, denom: f32) -> &'a mut Self {
        self.sig_num = num;
        self.sig_denom = denom;
        self
    }

    /// Set ticks per beat.
    pub fn with_ticks_per_beat<'a>(&'a mut self, ticks_per_beat: f64) -> &'a mut Self {
        self.ticks_per_beat = ticks_per_beat;
        self
    }

    /// Set bar start tick.
    pub fn with_bar_start_tick<'a>(&'a mut self, tick: f64) -> &'a mut Self {
        self.bar_start_tick = tick;
        self
    }

    /// Validate contents.
    pub fn validated<'a>(&'a self) -> std::result::Result<Self, Self> {
        if self.valid() {
            Ok(*self)
        } else {
            Err(*self)
        }
    }

    pub fn valid(&self) -> bool {
        !(self.bar < 1
            || self.beat < 1
            || self.sig_num <= 0.
            || self.sig_denom <= 0.
            || self.ticks_per_beat <= 0.
            || self.bpm < 0.
            || self.beat > self.sig_num.ceil() as _
            || self.tick >= self.ticks_per_beat.ceil() as _)
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

            bbt = Default::default();
            bbt.sig_num = 7.0;
            bbt.beat = 8;
            t(bbt);

            bbt = Default::default();
            bbt.ticks_per_beat = 96.0;
            bbt.tick = 96;
            t(bbt);
        }

        #[test]
        fn bbt_valid() {
            let mut p: TransportPosition = Default::default();
            let mut b: TransportBBT = Default::default();
            let mut i: TransportBBT = Default::default();
            i.beat = 5; //invalid

            assert!(!i.valid());

            assert!(!p.valid_bbt());
            assert_eq!(p.set_bbt(Some(b)), Ok(()));
            assert!(p.valid_bbt());
            assert_eq!(p.bbt(), Some(b));

            let mut t = |b: TransportBBT| {
                assert!(b.valid());
                assert_eq!(p.set_bbt(Some(b)), Ok(()));
                assert_eq!(p.bbt(), Some(b));
                //setting to something invalid keeps the old valid data
                assert_eq!(p.set_bbt(Some(i)), Err(i));
                assert_eq!(p.bbt(), Some(b));
            };

            for i in 1..10 {
                b.bar = i;
                t(b);
            }

            for i in 1..=4 {
                b.beat = i;
                t(b);
            }

            b.sig_num = 7.;
            for i in 1..=7 {
                b.beat = i;
                t(b);
            }

            b.beat = 1;
            b.sig_num = 4.;
            b.ticks_per_beat = 96.0;
            for i in 0..96 {
                b.tick = i;
                t(b);
            }

            for i in (10..300).step_by(7) {
                b.bpm = i as _;
                t(b);
            }
        }
    }
    mod bbt {
        use crate::TransportBBT;

        #[test]
        fn default() {
            let bbt: TransportBBT = Default::default();
            assert_eq!(bbt.bar, 1);
            assert_eq!(bbt.beat, 1);
            assert_eq!(bbt.tick, 0);
            assert_eq!(bbt.sig_num, 4.0);
            assert_eq!(bbt.sig_denom, 4.0);
            assert_eq!(bbt.ticks_per_beat, 1920.0);
            assert_eq!(bbt.bpm, 120.0);
            assert_eq!(bbt.bar_start_tick, 0.0);
        }

        #[test]
        fn builder_valid() {
            let mut bbt = TransportBBT::default();
            assert_eq!(
                TransportBBT::default().with_bbt(1, 1, 0).validated(),
                Ok(bbt)
            );

            bbt.bar = 100;
            bbt.beat = 2;
            bbt.tick = 230;
            assert_eq!(
                TransportBBT::default().with_bbt(100, 2, 230).validated(),
                Ok(bbt)
            );

            bbt = Default::default();
            bbt.sig_num = 7.0;
            bbt.sig_denom = 8.0;
            assert_eq!(
                TransportBBT::default().with_timesig(7.0, 8.0).validated(),
                Ok(bbt)
            );

            bbt = Default::default();
            bbt.ticks_per_beat = 2000.0;
            assert_eq!(
                TransportBBT::default()
                    .with_ticks_per_beat(2000.0)
                    .validated(),
                Ok(bbt)
            );

            bbt = Default::default();
            bbt.bar_start_tick = 1023.0;
            assert_eq!(
                TransportBBT::default()
                    .with_bar_start_tick(1023.0)
                    .validated(),
                Ok(bbt)
            );

            bbt = Default::default();
            bbt.bar = 2;
            bbt.beat = 3;
            bbt.tick = 20;
            bbt.bpm = 23.0;
            bbt.ticks_per_beat = 96.0;
            bbt.sig_num = 12.0;
            bbt.sig_denom = 5.0;
            bbt.bar_start_tick = 4.3;

            assert_eq!(
                TransportBBT::default()
                    .with_bbt(bbt.bar, bbt.beat, bbt.tick)
                    .with_bpm(bbt.bpm)
                    .with_ticks_per_beat(bbt.ticks_per_beat)
                    .with_timesig(bbt.sig_num, bbt.sig_denom)
                    .with_bar_start_tick(bbt.bar_start_tick)
                    .validated(),
                Ok(bbt)
            );

            //can simply use setters, could create invalid data..
            bbt = TransportBBT::default();
            bbt.with_bpm(120.0);
            assert_eq!(bbt.bpm, 120.0);
        }

        #[test]
        fn builder_invalid() {
            let mut bbt = TransportBBT::default();

            bbt.bpm = -1023.0;
            assert_eq!(
                TransportBBT::default().with_bpm(bbt.bpm).validated(),
                Err(bbt)
            );

            bbt = Default::default();
            bbt.bar = 0;
            assert_eq!(
                TransportBBT::default().with_bbt(0, 1, 0).validated(),
                Err(bbt)
            );

            bbt = Default::default();
            bbt.tick = bbt.ticks_per_beat as usize;
            assert_eq!(
                TransportBBT::default().with_bbt(1, 1, bbt.tick).validated(),
                Err(bbt)
            );

            for beat in &[0, 7] {
                bbt = Default::default();
                bbt.beat = *beat;
                assert_eq!(
                    TransportBBT::default().with_bbt(1, bbt.beat, 0).validated(),
                    Err(bbt)
                );
            }
        }
    }
}
