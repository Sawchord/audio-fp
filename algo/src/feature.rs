use alloc::collections::VecDeque;

use crate::{FrequencyFeature, Wavelet};

/// The feature finder is designed to find local  maxima in the amplitude
/// of a spectrogram.
/// It is designed as an online algorithm, such that it can run together with the frequencer.
/// It finds local maxima within a rectange, scoped by `t_span` in the time domain
/// and `f_span` in the frequency domain.
///
/// # Algorithm
/// The algorim keeps a slice of 2 * t_span of the spectogram.
/// For every frequency bin, it keeps a maximum value of the last 2 * t_span wavelets.
/// Then, it checks for maximas in the time domain at t_span.
pub struct FeatureFinder {
   block_size: usize,
   t_span: usize,
   //f_span: f64,
   time: usize,
   max_vals: Vec<FrequencyFeature>,
   val_window: Vec<VecDeque<FrequencyFeature>>,
}

impl FeatureFinder {
   pub fn new(block_size: usize, t_span: usize) -> Self {
      let proto_feature = FrequencyFeature {
         time: 0,
         bin_index: 0,
         frequency: 0.0,
         amplitude: 0.0,
      };

      let proto_line = VecDeque::from(vec![proto_feature.clone(); 2 * t_span]);

      // We only use half the blocksize
      let block_size = block_size / 2;

      Self {
         block_size,
         t_span,
         //f_span,
         time: 0,
         max_vals: vec![proto_feature; block_size],
         val_window: vec![proto_line; block_size],
      }
   }

   pub fn process(&mut self, wavelet: Wavelet) -> Vec<FrequencyFeature> {
      assert_eq!(wavelet.bins.len(), self.block_size);
      self.time += 1;

      // Update the lines and maxvals with he new wavlet
      for (max_val, (line, (bin_idx, bin))) in self.max_vals.iter_mut().zip(
         self
            .val_window
            .iter_mut()
            .zip(wavelet.bins.iter().enumerate()),
      ) {
         // Create a feature out of bin and append to line
         let feature = FrequencyFeature {
            time: self.time,
            bin_index: bin_idx,
            frequency: bin.frequency,
            amplitude: bin.amplitude,
         };

         // Update current max feature if needed
         if feature.amplitude > max_val.amplitude {
            *max_val = feature.clone();
         }

         line.push_back(feature);

         // Remove the old feature out of the line and update max_val if necessary
         let old_val = line.pop_front().unwrap();
         if &old_val == max_val {
            *max_val = FeatureFinder::get_max_in_line(&line).clone();
         }
      }

      // Now, we check, if each line has a local maximum at t_span
      let mut found_features = vec![];
      for val in self.max_vals.iter() {
         if val.time == self.time - self.t_span {
            // We have a local maximum in time.
            found_features.push(val.clone());
         }
      }

      found_features
   }

   fn get_max_in_line(line: &VecDeque<FrequencyFeature>) -> &FrequencyFeature {
      let mut max = &line[0];

      for elem in line.iter() {
         if elem.amplitude > max.amplitude {
            max = elem;
         }
      }

      max
   }
}
