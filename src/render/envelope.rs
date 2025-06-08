pub trait Envelope: Send + Sync {
    /// Returns a value from the envelope based on current time.
    fn value(&self, t: Float, duration: Float) -> Float;

    /// If the user needs to know how long the release will last.
    fn release_time(&self) -> Float;
}

pub struct ParametricDecayEnvelope {
    pub start: Float,
    pub end: Float,
    pub exponent: Float, // 1.0 = linear, >1 = exponential, <1 = log-like
}

impl Envelope for ParametricDecayEnvelope {
    fn value(&self, t: Float, duration: Float) -> Float {
        assert!(
            self.start != self.end,
            "Start and end values of envelope cannot
            be equal"
        );
        assert!(
            self.end < self.start,
            "Start time has to be before end time in decay envelope"
        );

        if t >= duration {
            return self.end;
        }

        let progress = t / duration;
        let shaped = (1.0 - progress).powf(self.exponent);
        let start = self.start;
        let end = self.end;
        end + (start - end) * shaped
    }

    fn release_time(&self) -> Float {
        0.0
    }
}

impl ParametricDecayEnvelope {
    fn reaches_threshold(&self, duration: Float, threshold: Float) -> bool {
        let end_val = self.value(duration, duration);
        (end_val - self.end).abs() <= threshold
    }
}

/// Duration represented in seconds.
pub struct ADSR {
    pub attack: Float,
    pub decay: Float,
    pub sustain: Float, // 0.0 - 1.0 value representing % of max volume
    pub release: Float,
}

impl Envelope for ADSR {
    fn value(&self, t: Float, note_duration: Float) -> Float {
        let release_start = note_duration;
        let end_time = release_start + self.release;

        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            let decay_t = t - self.attack;
            1.0 - (1.0 - self.sustain) * (decay_t / self.decay)
        } else if t < release_start {
            self.sustain
        } else if t < end_time {
            let release_t = t - release_start;
            self.sustain * (1.0 - release_t / self.release)
        } else {
            0.0
        }
    }

    /// The release time of the envelope will extend the notated duration of
    /// a given note. This function provides this extra time to the renderer
    /// for proper duration calculations.
    fn release_time(&self) -> Float {
        self.release
    }
}
