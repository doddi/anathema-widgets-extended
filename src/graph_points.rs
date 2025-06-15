use anathema::state::{List, Value};
use crate::graph::GraphSeriesState;


#[derive(Default)]
pub struct GraphData {
    pub series: Vec<GraphSeries>,
}

pub struct GraphSeries {
    pub points: Vec<u16>,
}

pub fn convert_series(graph_data: GraphData, y_range: u16) -> List<GraphSeriesState> {
    graph_data.series.iter().map(|series| {
        GraphSeriesState {
            points: convert_points(y_range, series)
        }
    }).collect()
}

fn convert_points(y_range: u16, series: &GraphSeries) -> Value<List<u16>> {
    let values: Vec<u16> = series.points.iter().map(|point| {
        // (*point / (y_range as u16)) as u16
        //TODO: just a 1:1 mapping at the moment to keep things simple
        *point
    }).collect();

    List::from_iter(values).into()
}

pub fn get_default_data_set() -> GraphData {
    let gs1 = GraphSeries {
        points: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    };
    let gs2 = GraphSeries {
        points: vec![1, 2],
    };
    // let series: Vec<GraphSeries> = vec![gs1, gs2];
    let series: Vec<GraphSeries> = vec![gs1];
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
    (smallest, largest)
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
