use anathema::state::{List, Value};
use crate::graph::{GraphDataState, GraphSeriesState};


#[derive(Default)]
pub struct GraphData {
    pub series: Vec<GraphSeries>,
}

pub struct GraphSeries {
    pub points: Vec<f32>,
}

// impl Into<GraphData> for GraphDataState {
//     fn into(self) -> GraphData {
//         GraphData {
//             series: self.series.to_ref().iter()
//                 .map(|s| s.into()).collect(),
//         }
//     }
// }
// 
// impl Into<GraphSeries> for GraphSeriesState {
//     fn into(self) -> GraphSeries {
//         GraphSeries {
//             points: self.points.to_ref().iter()
//                 .map(|point| point.copy_value()).collect()
//         }
//     }
// }

pub fn convert_series_to_state(graph_data: &GraphData, y_range: u16) -> List<GraphSeriesState> {
    let largest_range_in_series = determine_largest_range_in_series(graph_data);
    graph_data.series.iter().map(|series| {
        GraphSeriesState {
            points: convert_points(series, &largest_range_in_series, y_range)
        }
    }).collect()
}

fn convert_points(series: &GraphSeries, largest_range_in_series: &(f32, f32), y_range: u16) -> Value<List<u16>> {
    let range = largest_range_in_series.1 - largest_range_in_series.0;
    let values: Vec<u16> = series.points.iter().map(|point| {
        ((*point / range) * y_range as f32) as u16
    }).collect();

    List::from_iter(values).into()
}

pub fn get_default_data_set() -> GraphData {
    let gs1 = GraphSeries {
        points: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
    };
    let gs2 = GraphSeries {
        points: vec![1.0, 2.0],
    };
    // let series: Vec<GraphSeries> = vec![gs1, gs2];
    let series: Vec<GraphSeries> = vec![gs1];
    GraphData { series }
}

fn determine_largest_range_in_series(graph_data: &GraphData) -> (f32, f32) {
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
