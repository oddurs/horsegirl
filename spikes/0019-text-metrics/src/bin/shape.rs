//! Prints a label's advance from the shaped font.
fn main() {
    // The black box keeps a target with no arguments from folding the measure away.
    let text = std::hint::black_box(std::env::args().nth(1).unwrap_or_default());
    let font = text_metrics_spike::shaped::font();
    println!("{}", text_metrics_spike::shaped::width(&font, &text));
}
