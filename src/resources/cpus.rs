use anathema::component::{Children, Component, Context};
use anathema::state::{List, State, Value};

#[derive(Default)]
pub struct Cpus {}


#[derive(Default, State)]
pub struct CpusState {
    pub cpu_count: Value<u8>,
    pub cpu_usage: Value<List<f32>>,
}

pub struct CpusMessage {
    pub cpu_usage: Vec<f32>,
}

impl Component for Cpus {
    type State = CpusState;
    type Message = CpusMessage;

    fn on_message(&mut self, message: Self::Message, state: &mut Self::State, _children: Children<'_, '_>, _context: Context<'_, '_, Self::State>) {
        state.cpu_count.set(message.cpu_usage.len() as u8);
        state.cpu_usage.set(List::from_iter(message.cpu_usage));
    }
}