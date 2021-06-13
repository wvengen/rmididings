use rmidiproc::{FilterTrait,Discard};

pub struct Scene<'a> {
    pub name: &'a str,
    pub patch: &'a dyn FilterTrait,
    pub init: &'a dyn FilterTrait,
    pub exit: &'a dyn FilterTrait,
}

impl Scene<'_> {
    pub const DEFAULT: Self = Scene { name: "", patch: &Discard(), init: &Discard(), exit: &Discard() };

    pub fn default() -> Self {
        // TODO automatic naming (e.g. using a Cell)
        Scene { name: "", patch: &Discard(), init: &Discard(), exit: &Discard() }
    }
}

impl<'a> From<&'a dyn FilterTrait> for Scene<'a> {
    fn from(ft: &'a dyn FilterTrait) -> Scene<'a> {
        Scene { name: "Single patch", patch: ft, init: &Discard(), exit: &Discard() }
    }
}