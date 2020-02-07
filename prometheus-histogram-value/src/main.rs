use prometheus::{histogram_opts, Encoder, Histogram, Registry, TextEncoder};

fn main() {
    // Create a Histogram.
    let hist: Histogram = Histogram::with_opts(
        histogram_opts!(
            "example_histogram",
            "Used as an example",
            vec![0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 5.0]
        )
        .variable_label("example_variable"),
    )
    .unwrap();

    // Create a Registry and register Histogram.
    let r = Registry::new();
    // TODO: fix the panic which happens here
    r.register(Box::new(hist.clone())).unwrap();

    // Measure.
    let timer = hist.start_timer();
    timer.stop_and_record();

    // TODO: implement setting a value for the histogram

    // Gather the metrics.
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = r.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    // Output to the standard output.
    println!("{}", String::from_utf8(buffer).unwrap());
}
