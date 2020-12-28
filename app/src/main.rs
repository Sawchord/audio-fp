#![recursion_limit = "256"]

use yew::prelude::*;

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
