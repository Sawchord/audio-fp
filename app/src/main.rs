#![recursion_limit = "256"]
#![allow(dead_code)]

extern crate alloc;

mod display;
mod pipeline;

use core::time::Duration;
use js_sys::Error as JsError;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};
use yewtil::future::LinkFuture;

use crate::{
    display::{DisplayConfig, DisplayState},
    pipeline::Pipeline,
};

pub const STEP_SIZE: usize = 1024;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

fn js_err(value: JsValue, alt: &str) -> String {
    let value = match value.dyn_into::<JsError>() {
        Ok(value) => value,
        Err(_) => return alt.to_string(),
    };

    value.to_string().into()
}

enum Msg {
    PlayButtonPress,
    PipelineStarted,
    DisplayUpdate,
}

struct Model {
    link: ComponentLink<Self>,
    interval: Option<IntervalTask>,
    pipeline: Pipeline,
    started: bool,
    display: DisplayState,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let display = DisplayState::new(DisplayConfig {
            canvas_name: "display".to_string(),
            display_size: 600,
            display_height: 400,
        });

        let pipeline = Pipeline::new(display.sender()).unwrap();

        Self {
            link,
            interval: None,
            pipeline,
            started: false,
            display,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PlayButtonPress => {
                if self.started {
                    return false;
                }
                // Start pipeline and trigger PipelineStarted event afterwards
                let pipeline_clone = self.pipeline.clone();
                self.link.send_future(async move {
                    pipeline_clone.start().await.unwrap();
                    Msg::PipelineStarted
                });
                self.started = true;
                false
            }
            Msg::PipelineStarted => {
                // Start the display update interval
                self.interval = Some(IntervalService::spawn(
                    Duration::from_millis(500),
                    self.link.callback(|_| Msg::DisplayUpdate),
                ));
                false
            }
            // update the display
            Msg::DisplayUpdate => {
                self.display.update();
                false
            }
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn view(&self) -> Html {
        html! {
            <div>
                {"Audio FP"}
                <button onclick = self.link.callback(|_|Msg::PlayButtonPress)>
                    {"Start"}
                </button>
                <canvas id="display" style="width:1200;height:800;"/>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
