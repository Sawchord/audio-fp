use algo::{feature::FeatureFinder, frequencer::Frequencer};
use core::cell::RefCell;
use std::{rc::Rc, sync::mpsc::Sender};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
   AudioContext, AudioNode, AudioProcessingEvent, MediaStream, MediaStreamAudioSourceNode,
   MediaStreamAudioSourceOptions, MediaStreamConstraints, ScriptProcessorNode,
};

use crate::display::DisplayMessage;
use crate::{js_err, AppResult};

#[derive(Clone)]
pub struct Pipeline(Rc<RefCell<PipelineInner>>);

pub struct PipelineInner {
   audio_context: AudioContext,
   script_processor: ScriptProcessorNode,
   proc_pipeline: Option<AudioNode>,
   frequencer: Frequencer,
   feature_finder: FeatureFinder,
   display: Sender<DisplayMessage>,
}

impl Pipeline {
   pub fn new(display: Sender<DisplayMessage>) -> AppResult<Self> {
      let audio_context =
         AudioContext::new().map_err(|val| js_err(val, &"failed to establish audio context"))?;
      let script_processor = audio_context
         .create_script_processor_with_buffer_size(crate::STEP_SIZE as u32)
         .map_err(|val| js_err(val, &"failed to set up processing nodes"))?;

      let sample_rate = audio_context.sample_rate() as u32 as usize;
      web_sys::console::log_1(&JsValue::from_str(&format!(
         "Playback at samplerate {}",
         sample_rate
      )));

      Ok(Self(Rc::new(RefCell::new(PipelineInner {
         audio_context,
         script_processor,
         proc_pipeline: None,
         frequencer: Frequencer::new(sample_rate, crate::BLOCK_SIZE, crate::STEP_SIZE).unwrap(),
         feature_finder: FeatureFinder::new(crate::BLOCK_SIZE, crate::T_SPAN),
         display,
      }))))
   }

   pub async fn start(&self) -> AppResult<()> {
      let mut pipeline = self.0.borrow_mut();

      // Grab the media devices
      let media_devices = web_sys::window()
         .unwrap()
         .navigator()
         .media_devices()
         .map_err(|val| js_err(val, &"failed to grab media devices"))?;

      // Request audio access
      let media_stream = JsFuture::from(
         media_devices
            .get_user_media_with_constraints(MediaStreamConstraints::new().audio(&JsValue::TRUE))
            .unwrap(),
      )
      .await
      .map_err(|val| js_err(val, &"failed to acquire media stream"))?;

      let audio_src = MediaStreamAudioSourceNode::new(
         &pipeline.audio_context,
         &MediaStreamAudioSourceOptions::new(&MediaStream::unchecked_from_js(media_stream)),
      )
      .map_err(|val| js_err(val, &"failed to initialize audio source"))?;

      // Configure the audio callback
      let mut self_clone = self.clone();
      let audio_callback =
         Closure::wrap(Box::new(move |event| self_clone.process_audio_event(event))
            as Box<dyn FnMut(AudioProcessingEvent)>);

      // Set the audio callback into our processing node
      pipeline
         .script_processor
         .set_onaudioprocess(Some(audio_callback.as_ref().unchecked_ref()));
      audio_callback.forget();

      // connect audio src into the processing node
      let processing_pipeline = audio_src
         .connect_with_audio_node(&pipeline.script_processor)
         .map_err(|val| js_err(val, &"failed to set up pipeline"))?;
      pipeline.proc_pipeline = Some(processing_pipeline);

      Ok(())
   }

   fn process_audio_event(&mut self, event: AudioProcessingEvent) {
      let mut pipeline = self.0.borrow_mut();
      // Pull the audio out of the audio event
      let audio = event
         .input_buffer()
         .unwrap()
         .get_channel_data(0)
         .unwrap()
         .iter()
         .map(|x| *x as f64)
         .collect::<Vec<f64>>();
      assert_eq!(audio.len(), crate::STEP_SIZE);

      // Feed the data into the frequencer
      let wavelet = pipeline.frequencer.feed_audio(&audio);
      // Send the wavelet to the display
      pipeline
         .display
         .send(DisplayMessage::Wavelet(wavelet.clone()))
         .unwrap();

      let features = pipeline.feature_finder.process(wavelet);

      pipeline
         .display
         .send(DisplayMessage::Feature(features))
         .unwrap();

      // TODO: Further processing of the data
   }
}
