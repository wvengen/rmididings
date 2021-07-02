use rmididings_macros::*;
mod rmididings;
use rmididings::*;

fn main() {
    assert_debug_eq!(
        patch!(Pass() > Discard() > Pass() > Discard() > Pass()),
        "FilterChain(Chain, [Pass, Discard, Pass, Discard, Pass])"
    );
}