use rmididings_macros::*;
mod rmididings;
use rmididings::*;

fn main() {
    assert_debug_eq!(
        patch!(*Discard()),
        "FilterChain(Fork, [Pass, Discard])"
    )
}