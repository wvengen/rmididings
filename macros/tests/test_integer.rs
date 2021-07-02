use rmididings_macros::*;
mod rmididings;

fn main() {
    assert_debug_eq!(
        patch!(0),
        "0"
    );
}