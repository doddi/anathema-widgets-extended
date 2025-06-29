use std::time::Duration;
use anathema::component::{Children, Component, Context,State, Value};
use anathema::default_widgets::Canvas;
use anathema::geometry::{LocalPos, Size};
use anathema::state::Color;
use anathema::widgets::{Element, Style};

#[derive(State, Default)]
pub struct GraphDataState {
    pub point_width: Value<u16>,
    pub max_height: Value<u16>,
    pub min_height: Value<u16>,
    pub updated: Value<bool>,
}

#[derive(State, Default)]
pub struct Range {
    pub min: Value<u16>,
    pub max: Value<u16>,
}

#[derive(Default)]
pub struct Graph {
    pub graph_data: Option<GraphData>,
    pub range: (f32, f32),
}

#[derive(Default)]
pub struct GraphData {
    pub series: Vec<GraphSeries>,
}

#[derive(Default)]
pub struct GraphSeries {
    pub points: Vec<f32>,
}

impl Graph {
    fn draw_graph(&mut self, children: &mut Children, context: Context<GraphDataState>) {
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
                self.draw_data_points(canvas, &markers, &graph_type, size);
            });
    }

    fn calculate_state(&mut self, state: &mut GraphDataState, el: &mut Element) {
        let size = el.size();

        let mut max_width = 0;
        match &self.graph_data {
            None => {}
            Some(graph_data) => {
                for series in graph_data.series.iter() {
                    let point_width = calculate_point_width(series, size);
                    if point_width < max_width {
                        max_width = point_width;
                    }
                }
                state.point_width = Value::new(max_width);

                let largest_range_in_series = determine_largest_range_in_series(graph_data);
                state.max_height.set(largest_range_in_series.1 as u16);
                state.min_height.set(largest_range_in_series.0 as u16);
            }
        }
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
    fn clear_canvas(&self, canvas: &mut Canvas, size: Size) {
        for y in 0..size.height {
            for x in 0..size.width {
                canvas.put(' ', Style::reset(), LocalPos::new(x, y));
            }
        }
    }
    
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

    fn draw_data_points(&self, canvas: &mut Canvas, markers: &[char], graph_type: &GraphType, canvas_size: Size) {
        match &self.graph_data {
            None => {}
            Some(graph_data) => {
                let mut largest_points_len = 0;

                graph_data.series.iter().for_each(|series| {
                    (series.points.len() > largest_points_len)
                        .then(|| largest_points_len = series.points.len());
                });
                let mut bar_width = (canvas_size.width as usize / largest_points_len) as u16;
                if bar_width > 1 {
                    bar_width -= 1; // Ensure at least one character width for the bar
                }

                graph_data.series.iter().enumerate().for_each(|(index, series)| {
                    match graph_type {
                        GraphType::Point => self.draw_point_graph(bar_width, canvas_size, canvas, &series.points, Self::determine_marker(markers, index)),
                        GraphType::Bar => {
                            let mut style = Style::new();
                            style.set_bg(Self::get_bar_colour(index));
                            self.draw_bar_graph(bar_width, canvas_size, canvas, &series.points, style)
                        },
                    }
                });
            }
        }
    }

    fn determine_marker(markers: &[char], index: usize) -> char {
        if index > markers.len() {
            markers.first().unwrap().to_ascii_lowercase()
        } else {
            markers.get(index).unwrap().to_ascii_lowercase()
        }
    }

    fn draw_bar_graph(&self, bar_width: u16, canvas_size: Size, canvas: &mut Canvas, points: &[f32], style: Style) {
        let mut x = 1;

        points.iter().for_each(|point| {
            let converted_point = convert_point(canvas_size, self.range, point);
            for col in x..(x+bar_width) {
                for row in 0..converted_point {
                    canvas.put(' ', style, LocalPos::new(col, canvas_size.height - row));
                }
            }
            
            x = x + bar_width + 1; // +1 for the space between bars
        })
    }

    fn draw_point_graph(&self, point_width: u16, canvas_size: Size, canvas: &mut Canvas, points: &[f32], marker: char) {
        let mut x = 0;

        points.iter().for_each(| point| {
            let converted_point = convert_point(canvas_size, self.range, point);
            canvas.put(marker, Style::reset(), LocalPos::new(x, canvas_size.height - converted_point));
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
    type Message = ();

    fn on_tick(&mut self, state: &mut Self::State, mut children: Children<'_, '_>, context: Context<'_, '_, Self::State>, _dt: Duration) {
        let data = context.attributes.get("data").unwrap().as_list().unwrap();
        let mut graph_data: GraphData = GraphData::default();
        for series in data.iter() {
            let points = series.as_list().unwrap().iter()
                .map(|v| v.as_float().unwrap() as f32)
                .collect::<Vec<f32>>();
            graph_data.series.push(GraphSeries { points });
        }
        self.range = determine_largest_range_in_series(&graph_data);
        self.graph_data = Some(graph_data);

        children.elements().by_tag("canvas")
            .first(|el, _| {
                let size = el.size();
                self.clear_canvas(el.to::<Canvas>(), size);
                self.calculate_state(state, el);
            });

        self.draw_graph(&mut children, context);
    }

    fn on_resize(&mut self, state: &mut Self::State, _children: Children<'_, '_>, _context: Context<'_, '_, Self::State>) {
        state.updated.set(true);
    }
}
pub fn calculate_point_width(series: &GraphSeries, size: Size) -> u16 {
    let range = series.points.len() as f32;
    if range > 0.0 {
        (size.width as f32 / range) as u16
    } else {
        1 // Default to 1 if no points
    }
}

pub fn convert_point(size: Size, range: (f32, f32), point: &f32) -> u16 {
    ((*point / (range.1-range.0)) * size.height as f32) as u16
}

/// This function determines the smallest and largest values in the series of points
/// to be used for scaling the graph.
/// The idea is that it will eventually cater for negative values as well,
pub(crate) fn determine_largest_range_in_series(graph_data: &GraphData) -> (f32, f32) {
    let mut smallest: f32 = 0.0;
    let mut largest: f32 = 0.0;
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
    (smallest, largest)
}
