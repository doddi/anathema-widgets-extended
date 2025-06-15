use crate::ExtendedWidget;
use anathema::component::{Children, Component, Context, KeyCode, KeyEvent, List, State, Value};
use anathema::default_widgets::Canvas;
use anathema::geometry::LocalPos;
use anathema::runtime::Builder;
use anathema::state::Color;
use anathema::widgets::Style;
use crate::graph_points::{convert_series, get_default_data_set};

#[derive(State)]
pub struct GraphDataState {
    pub series: Value<List<GraphSeriesState>>,
}

impl GraphDataState {
    pub fn new() -> Self {
        Self {
            series: Default::default(),
        }
    }
}

#[derive(State)]
pub struct GraphSeriesState {
    pub points: Value<List<u16>>,
}

impl GraphSeriesState {
    pub(crate) fn new() -> Self {
        Self {
            points: Default::default(),
        }
    }
}

pub enum GraphMessage {
    // TODO: Data points are not send/sync so at the moment all I can think of is using json and then parse
    UpdateDataPoints(String),
}

pub struct Graph {
    x_range: (u16, u16),
    y_range: (u16, u16),
}

enum GraphType {
    Point, Bar
}

impl Into<GraphType> for &str {
    fn into(self) -> GraphType {
        match self {
            "bar" => GraphType::Bar,
            _ => GraphType::Point,
        }
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            x_range: (0, 0),
            y_range: (0, 0),
        }
    }
    
    fn get_y_range(&self) -> usize {
        /* TODO: instead of taking the values based on max ranges we should probably take into account
         that the graph area may be bigger than the actual data points, data points themselves might get resized
         */
        (self.y_range.1 - self.y_range.0) as usize
    }

    fn draw_axis(&self, _state: &GraphDataState, canvas: &mut Canvas, x_visible: bool, y_visible: bool, width: u16, height: u16) {
        if x_visible {
            // y position for the x axis is determined by the range of y
            let y = (height / (self.y_range.1 - self.y_range.0)) * self.y_range.1;
            for x in 0..width {
                canvas.put('_', Style::reset(), LocalPos::new(x, y));
            }
        }

        if y_visible {
            let x = (width / (self.x_range.1 - self.x_range.0)) * self.x_range.0;
            for y in self.y_range.0..self.y_range.1 {
                canvas.put('|', Style::reset(), LocalPos::new(x, y));
            }
        }
    }

    fn draw_data_points(&self, state: &mut GraphDataState, canvas: &mut Canvas, markers: &Vec<char>, graph_type: &GraphType) {
       
        let mut index = 0;
        state.series.for_each(|(series)| {
            match graph_type {
                GraphType::Point => self.draw_point_graph(canvas, series, Self::determine_marker(markers, index)),
                GraphType::Bar => {
                    let mut style = Style::new();
                    style.set_bg(Self::get_bar_colour(index));
                    self.draw_bar_graph(canvas, series, style)
                },
            }
        });
    }

    fn determine_marker(markers: &Vec<char>, index: usize) -> char {
        let marker: char;
        if index > markers.len() {
            marker = markers.get(0).unwrap().to_ascii_lowercase();
        } else {
            marker = markers.get(index).unwrap().to_ascii_lowercase();
        }
        marker
    }

    fn draw_bar_graph(&self, canvas: &mut Canvas, series: &mut GraphSeriesState, style: Style) {
        let mut x = 0;
        
        series.points.for_each(|point| {
            let value = *point;
            
            for next in 0..value {
                canvas.put(' ', style, LocalPos::new(x, (self.get_y_range() - next as usize) as u16));
            }
            x = x + 2;
        })
    }

    fn draw_point_graph(&self, canvas: &mut Canvas, series: &mut GraphSeriesState, marker: char) {
        let mut x = 0;

        series.points.for_each(|point| {
            canvas.put(marker, Style::reset(), LocalPos::new(x, self.get_y_range() as u16 - *point));
            x = x + 1;
        })
    }
    
    fn get_bar_colour(index: usize) -> Color {
       match index % 4 {
           0 => Color::Blue,
           1 => Color::Red,
           2 => Color::Green,
           _ => Color::Yellow,
       } 
    }
}

impl Component for Graph {
    type State = GraphDataState;
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

                let x_visible = context.attributes.get_as::<bool>("x_axis_visible")
                    .unwrap_or_else(|| true);
                let y_visible = context.attributes.get_as::<bool>("y_axis_visible")
                    .unwrap_or_else(|| true);
                let markers = context.attributes.get_as::<&str>("markers")
                    .unwrap_or_else(|| "@").chars().collect::<Vec<char>>();
                let graph_type: GraphType = context.attributes.get_as::<&str>("type")
                    .unwrap_or_else(|| "point").into();
                let width= context.attributes.get_as::<u16>("width")
                    .unwrap_or_else(|| 20);
                let height= context.attributes.get_as::<u16>("height")
                    .unwrap_or_else(|| 20);

                let list = convert_series(get_default_data_set(), height);
                
                //TODO: Why is this falling over?
                state.series.set(list);
                
                children.elements().by_tag("canvas")
                    .first(|el, _| {
                        let canvas = el.to::<Canvas>();
                        // self.draw_axis(state, canvas, x_visible, y_visible, width, height);
                        self.draw_data_points(state, canvas, &markers, &graph_type);
                    });
            },
        }
    }
}

impl ExtendedWidget for Graph {
    fn register(builder: &mut Builder<()>) {
        builder
            .prototype("graph", "templates/graph.aml", Graph::new, GraphDataState::new)
            .expect("Failed to register graph");
    }
}
