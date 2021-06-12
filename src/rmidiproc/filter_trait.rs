use event::*;
use event_stream::*;

// All filters implement this trait.
pub trait FilterTrait {
    // When adding or removing events, implement run.
    fn run(&self, evs: &mut EventStream) {
        evs.events.retain(|ev| self.filter_single(&ev));
        for ev in evs.events.iter_mut() {
            self.modify_single(ev);
        }
    }

    // When modifying single events, implement modify_single.
    fn modify_single(&self, _evs: &mut Event) {}

    // When selecting which events to keep and which not, implement filter_single.
    fn filter_single(&self, _evs: &Event) -> bool { true }
}