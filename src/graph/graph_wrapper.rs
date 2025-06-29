use std::time::Duration;
use anathema::component::{Children, Component, Context};
use anathema::resolver::ValueKind;
use crate::graph::graph::{GraphData, GraphSeries};

pub struct GraphWrapper {
    pub last_update: Option<Duration>,
    pub series: Option<GraphData>,
}

impl Component for GraphWrapper {
    type State = ();
    type Message = ();

    fn on_tick(&mut self, _state: &mut Self::State, mut children: Children<'_, '_>, context: Context<'_, '_, Self::State>, dt: Duration) {
        if let Some(last_update) = self.last_update {
            if dt.as_millis() - last_update.as_millis() < 1000 {
                return; // Skip update if less than 1 second has passed
            }
        }

        let series_count = context.attributes.get_as::<u8>("series_count").unwrap_or(1);
        let data_count = context.attributes.get_as::<u8>("data_count").unwrap_or(10);
        self.update_random_data(series_count, data_count);

        if let Some(data) = &self.series {
            children.components().by_name("graph")
                .first(|_, _, attrs| {
                    attrs.set("data", ValueKind::List(data.series.iter().map(|s| {
                        ValueKind::List(s.points.iter().map(|p| ValueKind::Float(*p as f64)).collect())
                    }).collect()));
                });
        }
    }
}

impl GraphWrapper {
    pub fn new() -> Self {
        Self {
            last_update: None,
            series: generate_random_series(1, 10),
        }
    }

    fn update_random_data(&mut self, series_count: u8, data_count: u8) {
        self.series = generate_random_series(series_count, data_count);
    }
}

fn generate_random_series(series_count: u8, data_count: u8) -> Option<GraphData> {
    let mut series = GraphData::default();
    for _ in 0..series_count {
        let mut data_points = GraphSeries::default();
        for _ in 0..data_count {
            data_points.points.push(rand::random::<f32>() * 100.0);
        }
        series.series.push(data_points);
    }
    Some(series)
}
