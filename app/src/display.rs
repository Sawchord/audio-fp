use algo::Wavelet;
use alloc::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use crate::AppResult;

#[derive(Debug, Clone)]
pub enum DisplayMessage {
   Wavelet(Wavelet),
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
            let red = (255.0 * amplitude) as usize;
            let red = if red < 255 { red as u8 } else { 255 };

            let img_index = 4 * (x + y * self.config.display_height);
            // Set red and alpha value
            img_data[img_index] = red;
            img_data[img_index + 3] = 255;
         }
      }

      let img_data = ImageData::new_with_u8_clamped_array_and_sh(
         Clamped(&mut img_data[..]),
         self.config.display_size as u32,
         self.config.display_height as u32,
      )
      .unwrap();

      let context = get_canvas(&self.config.canvas_name).unwrap();
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
         }
      }

      // Delete excess wavelets
      while self.wavelets.len() > self.config.display_size {
         self.wavelets.pop_front();
      }
   }
}

fn get_canvas(canvas_name: &str) -> AppResult<CanvasRenderingContext2d> {
   let document = web_sys::window().unwrap().document().unwrap();
   let canvas = document
      .get_element_by_id("canvas")
      .ok_or(format!("failed to grep canvas element {}", canvas_name))?;

   let canvas: HtmlCanvasElement = canvas
      .dyn_into::<HtmlCanvasElement>()
      .map_err(|_| "failed to convert element canvas element")?;

   let context = canvas
      .get_context(canvas_name)
      .unwrap()
      .unwrap()
      .dyn_into::<CanvasRenderingContext2d>()
      .unwrap();

   Ok(context)
}
