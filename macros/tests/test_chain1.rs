use rmididings_macros::*;
mod rmididings;
use rmididings::*;

fn main() {
    assert_debug_eq!(
        patch!(Pass() > Discard()),
        "FilterChain(Chain, [Pass, Discard])"
    );
}