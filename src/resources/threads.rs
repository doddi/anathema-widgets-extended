use anathema::component::{Children, Component, Context};
use anathema::state::{List, State, Value};

#[derive(Default)]
pub struct Threads {}


#[derive(Default, State)]
pub struct ThreadsState {
    pub thread_count: Value<u8>,
    pub threads: Value<List<f32>>,
}

pub struct ThreadMessage {
    pub thread_info: Vec<f32>,
}

impl Component for Threads {
    type State = ThreadsState;
    type Message = ThreadMessage;

    fn on_message(&mut self, message: Self::Message, state: &mut Self::State, children: Children<'_, '_>, context: Context<'_, '_, Self::State>) {
        state.thread_count.set(message.thread_info.len() as u8);
        state.threads.set(List::from_iter(message.thread_info));

        state.threads.set(List::from_iter(vec![1.0, 2.0, 3.0, 4.0, 5.0]));

        println!("{:?}", state.threads);
    }
}
