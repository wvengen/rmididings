use super::event_stream::*;

// All filters implement this trait.
pub trait FilterTrait {
    // When adding or removing events, implement run.
    fn run(&self, evs: &mut EventStream);

    // Only used for filters, where it is run when the filter is inside Not().
    // Override this if you make a filter that doesn't use define_filter!.
    fn run_inverse(&self, evs: &mut EventStream) { self.run(evs); }

    // Only used for Init filter
    fn run_init(&self, _evs: &mut EventStream) { }
    // Only used for Exit filter
    fn run_exit(&self, _evs: &mut EventStream) { }
}