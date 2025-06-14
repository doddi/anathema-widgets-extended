use crate::ExtendedWidget;
use anathema::component::{Children, Component, Context, KeyCode, KeyEvent, State};
use anathema::default_widgets::Canvas;
use anathema::geometry::LocalPos;
use anathema::runtime::Builder;
use anathema::state::Value;
use anathema::widgets::Style;

#[derive(State)]
pub struct GraphState {
    update_needed: Value<bool>,
}

#[derive(Default)]
pub struct GraphData {
    pub series: Vec<GraphSeries>,
}

pub struct GraphSeries {
    pub points: Vec<u16>,
}

impl GraphState {
    pub fn new() -> Self {
        GraphState {
            update_needed: Value::new(false),
        }
    }
}

pub enum GraphMessage {
    // TODO: Data points are not send/sync so at the moment all I can think of is using json and then parse
    UpdateDataPoints(String),
}

pub struct Graph {
    data: GraphData,
    x_range: (u16, u16),
    y_range: (u16, u16),
}

impl Graph {
    pub fn new() -> Self {
        Self {
            data: GraphData::default(),
            x_range: (0, 0),
            y_range: (0, 0),
        }
    }
    fn populate_graph_data_points(&mut self, state: &mut GraphState) {
        state.update_needed.set(true);
        self.data = get_default_data_set();
        self.y_range = determine_largest_range_in_series(&self.data);
        self.x_range = determine_largest_number_of_data_points(&self.data);
    }

    fn get_y_range(&self) -> u16 {
        /* TODO: instead of taking the values based on max ranges we should probably take into account
         that the graph area may be bigger than the actual data points, data points themselves might get resized
         */
       self.y_range.1 - self.y_range.0
    }

    fn draw_axis(&self, canvas: &mut Canvas, x_visible: bool, y_visible: bool) {
        if x_visible {
            for x in self.x_range.0..self.x_range.1 {
                canvas.put('_', Style::reset(), LocalPos::new(x, self.y_range.1));
            }
        }

        if y_visible {
            for y in self.y_range.0..self.y_range.1 {
                canvas.put('|', Style::reset(), LocalPos::new(0, y));
            }
        }
    }

    fn draw_data_points(&self, canvas: &mut Canvas, markers: &Vec<char>) {
        self.data.series.iter().enumerate().for_each(|(index, series)| {
            let mut x = 0;
            
            let marker: char;
            if index > markers.len() {
                marker = markers.get(0).unwrap().to_ascii_lowercase();
            } else {
                marker = markers.get(index).unwrap().to_ascii_lowercase();
            }
            
            series.points.iter().for_each(|point| {
                canvas.put(marker, Style::reset(), LocalPos::new(x, self.get_y_range() - *point as u16));
                x = x + 1;
            })
        })
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
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        match message {
            GraphMessage::UpdateDataPoints(_value) =>  {
                self.populate_graph_data_points(state);

                let x_visible = context.attributes.get_as::<bool>("x_axis_visible")
                    .unwrap_or_else(|| true);
                let y_visible = context.attributes.get_as::<bool>("y_axis_visible")
                    .unwrap_or_else(|| true);
                let markers = context.attributes.get_as::<&str>("markers")
                    .unwrap_or_else(|| "@").chars().collect::<Vec<char>>();
                
                children.elements().by_tag("canvas")
                    .first(|el, _| {
                        let canvas = el.to::<Canvas>();
                        self.draw_axis(canvas, x_visible, y_visible);
                        self.draw_data_points(canvas, &markers);
                    });
            },
        }
    }
}

fn get_default_data_set() -> GraphData {
    let gs1 = GraphSeries {
        points: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    };
    let gs2 = GraphSeries {
        points: vec![5, 6],
    };
    let series: Vec<GraphSeries> = vec![gs1, gs2];
    GraphData { series }
}

fn determine_largest_range_in_series(graph_data: &GraphData) -> (u16, u16) {
    let mut smallest: u16 = 0;
    let mut largest: u16 = 0;
    graph_data.series.iter().for_each(|series| {
        series.points.iter().for_each(|point| {
            if largest < *point {
                largest = *point;
            }
            if smallest > *point {
                smallest = *point;
            }
        })
    });
    (smallest as u16, largest as u16)
}

fn determine_largest_number_of_data_points(graph_data: &GraphData) -> (u16, u16) {
    let mut largest: u32 = 0;
    graph_data.series.iter().for_each(|series| {
        if largest < series.points.len() as u32 {
            largest = series.points.len() as u32;
        }
    });
    (0, largest as u16)
}

impl ExtendedWidget for Graph {
    fn register(builder: &mut Builder<()>) {
        builder
            .prototype("graph", "templates/graph.aml", Graph::new, GraphState::new)
            .expect("Failed to register graph");
    }
}
