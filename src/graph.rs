use crate::ExtendedWidget;
use anathema::component::{Children, Component, Context, KeyCode, KeyEvent, State};
use anathema::runtime::Builder;
use anathema::state::{List, Number, Value};

#[derive(State)]
pub struct GraphState {
    update_needed: Value<bool>,
    pub series: Value<GraphData>,
}

#[derive(State)]
pub struct GraphData {
    pub data: Value<List<GraphSeries>>,
    pub count: Value<u32>,
}

#[derive(State)]
pub struct GraphSeries {
    pub points: Value<List<u8>>,
    pub count: Value<u32>,
}

#[derive(State)]
pub struct DataPoint {
    pub x: Value<Number>,
    pub y: Value<Number>,
}

impl GraphState {
    pub fn new() -> Self {
        GraphState {
            update_needed: Value::new(false),
            series: Value::new(get_default_data_set()),
        }
    }
}

impl Default for DataPoint {
    fn default() -> Self {
        DataPoint {
            x: Value::new(0.into()),
            y: Value::new(0.into()),
        }
    }
}

pub enum GraphMessage {
    // TODO: Data points are not send/sync so at the moment all I can think of is using json and then parse
    UpdateDataPoints(String),
}

pub struct Graph {}

impl Graph {
    pub fn new() -> Self {
        Self {}
    }
}
impl Component for Graph {
    type State = GraphState;
    type Message = GraphMessage;

    fn on_key(
        &mut self,
        key: KeyEvent,
        _state: &mut Self::State,
        _children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        if key.code == KeyCode::Enter {
            context
                .components
                .by_name("graph")
                .send(GraphMessage::UpdateDataPoints("".to_string()))
        }
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _children: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
    ) {
        match message {
            GraphMessage::UpdateDataPoints(_value) => populate_graph_data_points(state),
        }
    }
}

fn get_default_data_set() -> GraphData {
    let gs1 = GraphSeries {
        points: List::from_iter(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).into(),
        count: 10.into(),
    };
    let gs2 = GraphSeries {
        points: List::from_iter(vec![11, 12]).into(),
        count: 2.into(),
    };
    let series: List<GraphSeries> = List::from_iter(vec![gs1, gs2]);
    GraphData {
        data: series.into(),
        count: 2.into(),
    }
}

fn populate_graph_data_points(state: &mut GraphState) {
    state.update_needed.set(true);
    state.series.set(get_default_data_set());
}

impl ExtendedWidget for Graph {
    fn register(builder: &mut Builder<()>) {
        builder
            .prototype("graph", "templates/graph.aml", Graph::new, GraphState::new)
            .expect("Failed to register graph");
    }
}
