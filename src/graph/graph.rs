use std::ops::Deref;
use std::time::Duration;
use crate::ExtendedWidget;
use anathema::component::{Children, Component, Context, KeyCode, KeyEvent, List, State, Value};
use anathema::default_widgets::Canvas;
use anathema::geometry::{LocalPos, Size};
use anathema::runtime::Builder;
use anathema::state::Color;
use anathema::widgets::{Element, Style};
use crate::graph::graph_points::{calculate_point_width, convert_series_to_state, get_default_data_set};

#[derive(State, Default)]
pub struct GraphDataState {
    pub point_width: Value<u16>,
    pub series: Value<List<GraphSeriesState>>,
    pub updated: Value<bool>,
}

#[derive(State)]
pub struct GraphSeriesState {
    pub points: Value<List<u16>>,
}

impl GraphSeriesState {}

#[derive(State, Default)]
pub struct Range {
    pub min: Value<u16>,
    pub max: Value<u16>,
}

pub enum GraphMessage {
    // TODO: Data points are not send/sync so at the moment all I can think of is using json and then parse
    UpdateDataPoints(String),
}

pub struct Graph {}

impl Graph {
    fn draw_graph(&mut self, state: &mut GraphDataState, children: &mut Children, context: Context<GraphDataState>) {
        let x_axis = context.attributes.get_as::<char>("x_axis");
        let y_axis = context.attributes.get_as::<char>("y_axis");
        let markers = context.attributes.get_as::<&str>("markers")
            .unwrap_or("@").chars().collect::<Vec<char>>();
        let graph_type: GraphType = context.attributes.get_as::<&str>("type")
            .unwrap_or("point").into();

        children.elements().by_tag("canvas")
            .first(|el, _| {
                let size = el.size();
                let canvas = el.to::<Canvas>();
                self.draw_axis(canvas, x_axis, y_axis, size);
                self.draw_data_points(state.deref(), canvas, &markers, &graph_type, size);
            });
    }

    fn calculate_state(&mut self, state: &mut GraphDataState, el: &mut Element) {
        let size = el.size();

        let mut max_width = 0;
        for series in get_default_data_set().series {
            let point_width = calculate_point_width(&series, size);
            if point_width > max_width {
                max_width = point_width;
            }
        }
        state.point_width = Value::new(max_width);

        let list = convert_series_to_state(&get_default_data_set(), size);
        state.series.set(list);
        state.updated.set(true);
    }
}

enum GraphType {
    Point, Bar
}

impl From<&str> for GraphType {
    fn from(value: &str) -> Self {
        match value {
            "bar" => GraphType::Bar,
            _ => GraphType::Point,
        }
    }
}

impl Graph {

    fn draw_axis(&self, canvas: &mut Canvas, x_axis: Option<char>, y_axis: Option<char>, size: Size) {
        match x_axis {
            None => {}
            Some(value) => {
                for x in 0..size.width {
                    canvas.put(value, Style::reset(), LocalPos::new(x, size.height - 1));
                }
            }
        }
        
        match y_axis {
            None => {}
            Some(value) => {
                for y in 0..size.height {
                    canvas.put(value, Style::reset(), LocalPos::new(0, y));
                }
            }
        }
    }

    //TODO: This should take into account the canvas width and draw the data points accordingly
    fn draw_data_points(&self, state: &GraphDataState, canvas: &mut Canvas, markers: &[char], graph_type: &GraphType, size: Size) {

        let mut largest_points_len = 0;
        state.series.to_ref().iter().for_each(|series| {
            (series.to_ref().points.len() > largest_points_len)
                .then(|| largest_points_len = series.to_ref().points.len());
        });
        let mut bar_width = (size.width as usize / largest_points_len) as u16;
        if bar_width > 1 {
            bar_width -= 1; // Ensure at least one character width for the bar
        }
        
        state.series.to_ref().iter().enumerate().for_each(|(index, series)| {
            match graph_type {
                GraphType::Point => self.draw_point_graph(bar_width, size.height, canvas, &series.to_ref().points, Self::determine_marker(markers, index)),
                GraphType::Bar => {
                    let mut style = Style::new();
                    style.set_bg(Self::get_bar_colour(index));
                    self.draw_bar_graph(bar_width, size.height, canvas, &series.to_ref().points, style)
                },
            }
        });
    }

    fn determine_marker(markers: &[char], index: usize) -> char {
        if index > markers.len() {
            markers.first().unwrap().to_ascii_lowercase()
        } else {
            markers.get(index).unwrap().to_ascii_lowercase()
        }
    }

    fn draw_bar_graph(&self, bar_width: u16, y_range: u16, canvas: &mut Canvas, points: &Value<List<u16>>, style: Style) {
        let mut x = 1;

        points.to_ref().iter().for_each(|point| {
            let value = point.copy_value();

            for col in x..(x+bar_width) {
                for row in 0..value {
                    canvas.put(' ', style, LocalPos::new(col, y_range - row));
                }
            }
            
            x = x + bar_width + 1; // +1 for the space between bars
        })
    }

    fn draw_point_graph(&self, point_width: u16, y_range: u16, canvas: &mut Canvas, points: &Value<List<u16>>, marker: char) {
        let mut x = 0;

        points.to_ref().iter().for_each(| point| {
            canvas.put(marker, Style::reset(), LocalPos::new(x, y_range - point.copy_value()));
            x += point_width + 1; // +1 for the space between points
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

    fn on_tick(&mut self, state: &mut Self::State, mut children: Children<'_, '_>, context: Context<'_, '_, Self::State>, _dt: Duration) {
        if state.updated.copy_value() {
            state.updated.set(false);
            self.draw_graph(state, &mut children, context);
        }
    }



    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
    ) {
        match message {
            GraphMessage::UpdateDataPoints(_value) =>  {
                children.elements().by_tag("canvas")
                    .first(|el, _| {
                        self.calculate_state(state, el);
                    });

            },
        }
    }

    fn on_resize(&mut self, state: &mut Self::State, mut children: Children<'_, '_>, _context: Context<'_, '_, Self::State>) {
        // TODO: Why is this not working?
        children.elements().by_tag("canvas")
            .first(|el, _| {
                self.calculate_state(state, el);
            });
    }
}

impl ExtendedWidget for Graph {
    fn register(builder: &mut Builder<()>) {
        builder.component("graph", "templates/graph.aml", Graph {}, GraphDataState::default())
            .expect("Failed to register graph component");
    }
}
