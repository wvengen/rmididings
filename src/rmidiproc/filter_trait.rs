use event_stream::*;

// All filters implement this trait.
pub trait FilterTrait {
    // When adding or removing events, implement run.
    fn run(&self, evs: &mut EventStream);

    // Only used for Init filter
    fn run_init(&self, _evs: &mut EventStream) { }
    // Only used for Exit filter
    fn run_exit(&self, _evs: &mut EventStream) { }
}