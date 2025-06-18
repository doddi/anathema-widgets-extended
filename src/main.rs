mod graph;
mod graph_points;
mod starfield;

use anathema::prelude::{Backend, Document, TuiBackend};
use anathema::runtime::{Builder, Runtime};

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

    graph::Graph::register(&mut builder);
    starfield::Starfield::register(&mut builder);

    builder
        .finish(|runtime| runtime.run(&mut backend))
        .unwrap();
}

pub(crate) trait ExtendedWidget {
    fn register(builder: &mut Builder<()>);
}

