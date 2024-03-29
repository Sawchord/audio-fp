use algo::{FrequencyFeature, Wavelet};
use alloc::collections::VecDeque;
use std::{
   collections::HashMap,
   sync::mpsc::{channel, Receiver, Sender},
};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use crate::AppResult;

#[derive(Debug, Clone)]
pub enum DisplayMessage {
   Wavelet(Wavelet),
   Feature(Vec<FrequencyFeature>),
}

#[derive(Debug, Clone)]
pub struct DisplayConfig {
   pub canvas_name: String,
   pub display_size: usize,
   pub display_height: usize,
}

#[derive(Debug)]
pub struct DisplayState {
   config: DisplayConfig,
   tx: Sender<DisplayMessage>,
   rx: Receiver<DisplayMessage>,
   time: usize,
   wavelets: VecDeque<Wavelet>,
   features: HashMap<usize, FrequencyFeature>,
}

impl DisplayState {
   pub fn new(config: DisplayConfig) -> Self {
      let (tx, rx) = channel();

      Self {
         config,
         tx,
         rx,
         time: 0,
         wavelets: VecDeque::new(),
         features: HashMap::new(),
      }
   }

   pub fn sender(&self) -> Sender<DisplayMessage> {
      self.tx.clone()
   }

   pub fn update(&mut self) {
      // Read out msg buffer and get rid of excess data
      self.read_msgs();

      // Create the image data out of the wavelet data
      let mut img_data = vec![0; 4 * self.config.display_size * self.config.display_height];
      for (x, wavelet) in self.wavelets.iter().enumerate() {
         for y in 0..self.config.display_height {
            // Retreive the correct frequency bin
            let index = y * (wavelet.bins.len() / self.config.display_height);
            let amplitude = wavelet
               .bins
               .get(index)
               .map(|bin| bin.amplitude)
               .unwrap_or(0.0);

            // Clamp down the red value
            // Fix 1.0 at 127 and infinity at 255
            let red = 255f64 * (2f64.powf(-amplitude)) * (2f64.powf(amplitude) - 1f64);
            let red = if red < 255f64 { red as u8 } else { 255 };

            let img_index = 4 * (x + y * self.config.display_size);
            // Set red and alpha value
            img_data[img_index] = red;
            img_data[img_index + 3] = 255;
         }
      }

      web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
         "Displaying {} features",
         self.features.len()
      )));

      // Paint features green
      for (time, feature) in self.features.iter() {
         let x = match self.time > self.config.display_size {
            true => self.config.display_size - (self.time - time),
            false => *time,
         };
         let y = feature.bin_index * self.config.display_height / (crate::BLOCK_SIZE / 2);
         let img_index = 4 * (x + y * self.config.display_size);

         // Paint the pixel under the feature green
         let amplitude = feature.amplitude;
         let green = 255f64 * (2f64.powf(-amplitude)) * (2f64.powf(amplitude) - 1f64);
         let green = if green < 255f64 { green as u8 } else { 255 };

         img_data[img_index] = 0;
         img_data[img_index + 1] = green;
      }

      let img_data = ImageData::new_with_u8_clamped_array_and_sh(
         Clamped(&mut img_data[..]),
         self.config.display_size as u32,
         self.config.display_height as u32,
      )
      .unwrap();

      let context = self.get_canvas().unwrap();
      context.put_image_data(&img_data, 0.0, 0.0).unwrap();
   }

   fn read_msgs(&mut self) {
      // Read out the msg buffer and update the state
      while let Ok(msg) = self.rx.try_recv() {
         match msg {
            DisplayMessage::Wavelet(wavelet) => {
               self.time += 1;
               self.wavelets.push_back(wavelet);
            }
            DisplayMessage::Feature(features) => {
               for feature in features {
                  self.features.insert(feature.time, feature);
               }
            }
         }
      }

      // Delete excess wavelets
      while self.wavelets.len() > self.config.display_size {
         self.wavelets.pop_front();
      }

      // Keep only features that are not outdated
      let outdated = match self.time > self.config.display_size {
         true => self.time - self.config.display_size,
         false => 0,
      };
      self.features.retain(|time, _| *time >= outdated);
   }

   fn get_canvas(&self) -> AppResult<CanvasRenderingContext2d> {
      // Get the canvas html element
      let document = web_sys::window().unwrap().document().unwrap();
      let canvas = document
         .get_element_by_id(&self.config.canvas_name)
         .ok_or(format!(
            "failed to grab canvas element {}",
            self.config.canvas_name
         ))?;
      let canvas: HtmlCanvasElement = canvas
         .dyn_into::<HtmlCanvasElement>()
         .map_err(|_| "failed to convert element canvas element")?;

      // Set the size correctly
      canvas.set_width(self.config.display_size as u32);
      canvas.set_height(self.config.display_height as u32);

      // Get the canvas context to draw on
      let context = canvas
         .get_context("2d")
         .unwrap()
         .unwrap()
         .dyn_into::<CanvasRenderingContext2d>()
         .unwrap();
      Ok(context)
   }
}
