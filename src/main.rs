mod graph;
mod starfield;
mod resources;

use std::thread;
use std::time::Duration;
use anathema::component::{ComponentId, Emitter};
use anathema::prelude::{Backend, Document, TuiBackend};
use anathema::runtime::Runtime;
use sysinfo::System;
use graph::graph_wrapper::GraphWrapper;
use crate::resources::cpus::CpusMessage;

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
    let thread_id = builder.component("cpus", "templates/resources/cpus.aml", resources::cpus::Cpus::default(), resources::cpus::CpusState::default()).unwrap();

    run_thread(builder.emitter(), thread_id);

    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();
}

fn run_thread(emitter: Emitter, thread_id: ComponentId<CpusMessage>) {
    thread::spawn(move || {
        let mut system = System::new_all();

        loop {
            system.refresh_cpu_usage();

            let thread_info: Vec<f32> = system.cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect();

            let _ = emitter.emit(thread_id, CpusMessage {
                cpu_usage: thread_info,
            });
            thread::sleep(Duration::from_millis(100));
        }
    });
}
