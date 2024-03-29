extern crate alloc;

pub mod feature;
pub mod frequencer;

#[derive(Debug, Clone)]
pub struct FrequencyBin {
    pub amplitude: f64,
    pub frequency: f64,
}

#[derive(Debug, Clone)]
pub struct Wavelet {
    pub bins: Vec<FrequencyBin>,
}

impl Wavelet {
    pub fn empty(frames: usize) -> Self {
        Wavelet {
            bins: (0..frames)
                .map(|_| FrequencyBin {
                    amplitude: 0.0,
                    frequency: 0.0,
                })
                .collect::<Vec<_>>(),
        }
    }

    // TODO: Make this fancy with iterators
    pub fn base_freq(&self) -> f64 {
        let mut max_freq = 0.0;
        let mut max_amp = 0.0;
        for bin in &self.bins {
            if bin.amplitude > max_amp {
                max_amp = bin.amplitude;
                max_freq = bin.frequency;
            }
        }
        max_freq
        //let max = self.bins.iter().max();
    }

    pub fn pitch_shift(&mut self, pitch_shift: f64) {
        let bins = &self.bins;

        // Create empty bins
        let mut new_bins = core::iter::repeat(FrequencyBin {
            amplitude: 0.0,
            frequency: 0.0,
        })
        .take(bins.len())
        .collect::<Vec<_>>();

        for k in 0..bins.len() {
            let index = ((k as f64) * pitch_shift) as usize;

            if index < new_bins.len() {
                new_bins[index].amplitude += bins[k].amplitude;
                new_bins[index].frequency = bins[k].frequency * pitch_shift
            }
        }

        self.bins = new_bins;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrequencyFeature {
    pub time: usize,
    pub bin_index: usize,
    pub frequency: f64,
    pub amplitude: f64,
}
