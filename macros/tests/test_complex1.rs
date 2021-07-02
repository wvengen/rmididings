use rmididings_macros::*;
mod rmididings;
use rmididings::*;

fn main() {
    assert_debug_eq!(
        patch!(
            Pass() > Pass() > [
                Discard(),
                Discard(),
                Pass() > Discard() > Pass() > Discard() > [Pass(), Discard()],
                Pass() / Pass() / Discard(),
                (Pass() > Discard() > (Pass() / Discard()))
            ]
        ),
        concat!(
            "FilterChain(Chain, [Pass, Pass, FilterChain(Fork, [",
                "Discard, ",
                "Discard, ",
                "FilterChain(Chain, [Pass, Discard, Pass, Discard, FilterChain(Fork, [Pass, Discard])])",
                "Pass, Pass, Discard, ",
                "FilterChain(Chain, [Pass, Discard, FilterChain(Fork, [Pass, Discard]))",
                "]))"
        )
    );
}