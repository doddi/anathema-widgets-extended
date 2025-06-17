use std::ops::Deref;
use std::time::Duration;
use crate::ExtendedWidget;
use anathema::component::{Children, Component, Context, KeyCode, KeyEvent, List, State, Value};
use anathema::default_widgets::Canvas;
use anathema::geometry::LocalPos;
use anathema::runtime::Builder;
use anathema::state::Color;
use anathema::widgets::Style;
use crate::graph_points::{convert_series_to_state, get_default_data_set};

#[derive(State, Default)]
pub struct GraphDataState {
    pub x_range: Value<Range>,
    pub y_range: Value<Range>,
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
        let x_visible = context.attributes.get_as::<bool>("x_axis_visible")
            .unwrap_or(true);
        let y_visible = context.attributes.get_as::<bool>("y_axis_visible")
            .unwrap_or(true);
        let markers = context.attributes.get_as::<&str>("markers")
            .unwrap_or("@").chars().collect::<Vec<char>>();
        let graph_type: GraphType = context.attributes.get_as::<&str>("type")
            .unwrap_or("point").into();
        let width = context.attributes.get_as::<u16>("width")
            .unwrap_or(20);
        let height = context.attributes.get_as::<u16>("height")
            .unwrap_or(20);


        let s = state.deref();;

        children.elements().by_tag("canvas")
            .first(|el, _| {
                let canvas = el.to::<Canvas>();
                self.draw_axis(s, canvas, x_visible, y_visible, width, height);
                self.draw_data_points(s, canvas, &markers, &graph_type, width);
            });
    }
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

    fn determine_ranges(&self, graph_data: &mut GraphDataState, width: u16, height: u16) {
        let mut y_range = (0u16, 0u16);
        let mut x_range = (0u16, 0u16);

        graph_data.series.for_each(|series| {
            if series.points.len() > x_range.1 as usize {
                x_range.1 = series.points.len() as u16;
            }

            series.points.for_each(|point| {
                if *point > y_range.1 {
                    y_range.1 = *point;
                }
            })
        });
        graph_data.y_range.set(Range { min: Value::new(y_range.0), max: Value::new(y_range.1) });
        graph_data.x_range.set(Range { min: Value::new(x_range.0), max: Value::new(x_range.1) });
    }

    fn draw_axis(&self, state: &GraphDataState, canvas: &mut Canvas, x_visible: bool, y_visible: bool, width: u16, height: u16) {
        if x_visible {
            let y_range = state.y_range.to_ref();
            // y position for the x axis is determined by the range of y
            let y = (height / (y_range.max.copy_value() - y_range.min.copy_value())) * y_range.max.copy_value() - 1;
            for x in 0..width {
                canvas.put('▁', Style::reset(), LocalPos::new(x, y));
            }
        }

        if y_visible {
            let x_range = state.x_range.to_ref();
            let y_range = state.y_range.to_ref();
            let i = (x_range.max.copy_value() - x_range.min.copy_value());
            let i1 = x_range.min.copy_value();
            let x = (width / i) * i1;
            for y in y_range.min.copy_value()..y_range.max.copy_value() {
                canvas.put('│', Style::reset(), LocalPos::new(x, y));
            }
        }
    }

    //TODO: This should take into account the canvas width and draw the data points accordingly
    fn draw_data_points(&self, state: &GraphDataState, canvas: &mut Canvas, markers: &[char], graph_type: &GraphType, width: u16) {
        let y_range = state.y_range.to_ref().max.copy_value() - state.y_range.to_ref().min.copy_value();
        // let x_range = state.x_range.to_ref().max.copy_value() - state.x_range.to_ref().min.copy_value();

        state.series.to_ref().iter().enumerate().for_each(|(index, series)| {
            match graph_type {
                GraphType::Point => self.draw_point_graph(y_range, canvas, &series.to_ref().points, Self::determine_marker(markers, 0)),
                GraphType::Bar => {
                    let mut style = Style::new();
                    style.set_bg(Self::get_bar_colour(index));
                    self.draw_bar_graph(width, y_range, canvas, &series.to_ref().points, style)
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

    fn draw_bar_graph(&self, x_range: u16, y_range: u16, canvas: &mut Canvas, points: &Value<List<u16>>, style: Style) {
        let mut x = 0;
        //TODO: Bad, need to look at a better way to determine the width of the bar
        let mut bar_width = (x_range as usize / points.len()) as u16;
        if bar_width > 1 {
            bar_width -= 1; // Ensure at least one character width for the bar
        }

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

    fn draw_point_graph(&self, range: u16, canvas: &mut Canvas, points: &Value<List<u16>>, marker: char) {
        let mut x = 0;

        points.to_ref().iter().for_each(| point| {
            canvas.put(marker, Style::reset(), LocalPos::new(x, range - point.copy_value()));
            x += 1;
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

    fn on_tick(&mut self, state: &mut Self::State, mut children: Children<'_, '_>, context: Context<'_, '_, Self::State>, dt: Duration) {
        if state.updated.copy_value() {
            self.draw_graph(state, &mut children, context);
        }
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        match message {
            GraphMessage::UpdateDataPoints(_value) =>  {
                let width = context.attributes.get_as::<u16>("width")
                    .unwrap_or(20);
                let height = context.attributes.get_as::<u16>("height")
                    .unwrap_or(20);

                let list = convert_series_to_state(&get_default_data_set(), height);
                state.series.set(list);

                self.determine_ranges(state, width, height);
                state.updated.set(true);
            },
        }
    }
}

impl ExtendedWidget for Graph {
    fn register(builder: &mut Builder<()>) {
        builder.component("graph", "templates/graph.aml", Graph {}, GraphDataState::default())
            .expect("Failed to register graph component");
    }
}
