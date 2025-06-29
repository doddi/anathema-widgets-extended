mod graph;
mod starfield;

use anathema::prelude::{Backend, Document, TuiBackend};
use anathema::runtime::Runtime;
use graph::graph_wrapper::GraphWrapper;

fn main() {
    let doc = Document::new("@index");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();
    backend.finalize();

    let mut builder = Runtime::builder(doc, &backend);
    builder.fps(10);
    builder
        .default::<()>("index", "templates/index.aml")
        .unwrap();

    builder.component("starfield", "templates/starfield.aml", starfield::starfield::Starfield::default(), starfield::starfield::StarfieldState::default()).unwrap();
    builder.prototype("graph", "templates/graph.aml", graph::graph::Graph::default, graph::graph::GraphDataState::default).unwrap();
    builder.component("graph_wrapper", "templates/graph_wrapper.aml", GraphWrapper::new(), ()).unwrap();

    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();
}

