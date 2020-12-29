#![recursion_limit = "256"]
#![allow(dead_code)]

extern crate alloc;

mod display;
mod pipeline;

use yew::prelude::*;

pub const STEP_SIZE: usize = 1000;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn view(&self) -> Html {
        html! {
            <div>
                {"Audio FP"}
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
