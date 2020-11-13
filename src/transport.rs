use jack_sys as j;

pub struct Transport {
    client_ptr: *mut j::jack_client_t,
}

#[repr(transparent)]
pub struct TransportPosition(j::jack_position_t);

#[derive(Debug, Clone, Copy)]
pub enum TransportState {
    Stopped,
    Rolling,
    Starting,
}

pub struct TransportStatePosition {
    pub pos: TransportPosition,
    pub state: TransportState,
}

pub struct TransportBBT {
    /// Time signature bar, 1 or more.
    pub bar: usize,
    /// Time signature beat, 1 and less than sig_num.
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
    /// Start the JACK transport rolling.
    ///
    /// # Remarks
    ///
    /// * Any client can make this request at any time.
    /// * It takes effect no sooner than the next process cycle, perhaps later if there are
    /// slow-sync clients.
    /// * This function is realtime-safe.
    pub fn start(&self) {
        unsafe {
            j::jack_transport_start(self.client_ptr);
        }
    }

    /// Stop the JACK transport.
    ///
    /// # Remarks
    ///
    /// * Any client can make this request at any time.
    /// * It takes effect on the next process cycle.
    /// * This function is realtime-safe.
    pub fn stop(&self) {
        unsafe {
            j::jack_transport_stop(self.client_ptr);
        }
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
    pub fn reposition(&self, pos: &TransportPosition) -> Result<(), ()> {
        Self::to_result(unsafe {
            j::jack_transport_reposition(
                self.client_ptr,
                std::mem::transmute::<&TransportPosition, *const j::jack_position_t>(pos),
            )
        })
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
    pub fn locate(&self, frame: crate::Frames) -> Result<(), ()> {
        Self::to_result(unsafe { j::jack_transport_locate(self.client_ptr, frame) })
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
    fn to_result(v: ::libc::c_int) -> Result<(), ()> {
        if v == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Query the current transport state and position.
    ///
    /// # Remarks
    ///
    /// * This function is realtime-safe, and can be called from any thread.
    /// * If called from the process thread, `pos` corresponds to the first frame of the current cycle and the state returned is valid for the entire cycle.
    pub fn query(&self) -> TransportStatePosition {
        let mut pos: std::mem::MaybeUninit<TransportPosition> = std::mem::MaybeUninit::zeroed();
        let state = Self::to_state(unsafe {
            j::jack_transport_query(
                self.client_ptr,
                std::mem::transmute::<*mut TransportPosition, *mut j::jack_position_t>(
                    pos.as_mut_ptr(),
                ),
            )
        });
        TransportStatePosition {
            pos: unsafe { pos.assume_init() },
            state,
        }
    }

    /// Query the current transport state.
    ///
    /// # Remarks
    ///
    /// * This function is realtime-safe, and can be called from any thread.
    /// * If called from the process thread, the state returned is valid for the entire cycle.
    pub fn query_state(&self) -> TransportState {
        Self::to_state(unsafe { j::jack_transport_query(self.client_ptr, std::ptr::null_mut()) })
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
    pub fn frame(&self) -> crate::Frames {
        self.0.frame
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

    /// Get the frame offset for the BBT fields.
    ///
    /// # Remarks
    ///
    /// * Should be assumed to be 0 if `None`.
    /// * If this value is Some(0), the bbt time refers to the first frame of this cycle.
    /// * If the value is positive, the bbt time refers to a frame that many frames **before** the start of the cycle.
    pub fn bbt_offset(&self) -> Option<crate::Frames> {
        if self.valid_bbt_frame_offset() {
            Some(self.0.bbt_offset)
        } else {
            None
        }
    }
}
