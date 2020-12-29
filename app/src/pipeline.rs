use crate::AppResult;
use algo::frequencer::Frequencer;
use core::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
   AudioContext, AudioNode, AudioProcessingEvent, MediaStream, MediaStreamAudioSourceNode,
   MediaStreamAudioSourceOptions, MediaStreamConstraints, ScriptProcessorNode,
};

#[derive(Clone)]
pub struct Pipeline(Rc<RefCell<PipelineInner>>);

pub struct PipelineInner {
   audio_context: AudioContext,
   script_processor: ScriptProcessorNode,
   proc_pipeline: Option<AudioNode>,
   frequencer: Frequencer,
}

impl Pipeline {
   pub fn new() -> AppResult<Self> {
      let audio_context = AudioContext::new().map_err(|_| "failed to establish audio context")?;
      let script_processor = audio_context
         .create_script_processor_with_buffer_size(crate::STEP_SIZE as u32)
         .map_err(|_| "failed to set up processing nodes")?;

      Ok(Self(Rc::new(RefCell::new(PipelineInner {
         audio_context,
         script_processor,
         proc_pipeline: None,
         frequencer: Frequencer::new(48000, 4096, 1024).unwrap(),
      }))))
   }

   pub async fn start(&self) -> AppResult<()> {
      let mut pipeline = self.0.borrow_mut();

      // Grab the media devices
      let media_devices = web_sys::window()
         .unwrap()
         .navigator()
         .media_devices()
         .map_err(|_| "failed to grab media devices")?;

      // Request audio access
      let media_stream = JsFuture::from(
         media_devices
            .get_user_media_with_constraints(MediaStreamConstraints::new().audio(&JsValue::TRUE))
            .unwrap(),
      )
      .await
      .map_err(|_| "failed to acquire media stream")?;

      let audio_src = MediaStreamAudioSourceNode::new(
         &pipeline.audio_context,
         &MediaStreamAudioSourceOptions::new(&MediaStream::unchecked_from_js(media_stream)),
      )
      .map_err(|_| "failed to initialize audio source")?;

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
         .map_err(|_| "failed to set up pipeline")?;
      pipeline.proc_pipeline = Some(processing_pipeline);

      Ok(())
   }

   fn process_audio_event(&mut self, _event: AudioProcessingEvent) {
      todo!()
   }
}
