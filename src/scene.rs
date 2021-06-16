use super::proc::{FilterTrait,Discard,SceneNum};

pub struct Scene<'a> {
    pub name: &'a str,
    pub patch: &'a dyn FilterTrait,
    pub init: &'a dyn FilterTrait,
    pub exit: &'a dyn FilterTrait,
    pub subscenes: &'a [&'a Scene<'a>],
}

impl Scene<'_> {
    pub const DEFAULT: Self = Scene { name: "", patch: &Discard(), init: &Discard(), exit: &Discard(), subscenes: &[] };

    pub fn default() -> Self {
        // TODO automatic naming (e.g. using a Cell)
        Self::DEFAULT
    }

    pub fn get_subscene(&self, subscene_num: SceneNum) -> Option<&Scene> {
        if self.subscenes.len() > subscene_num as usize {
            Some(self.subscenes[subscene_num as usize])
        } else {
            None
        }
    }

    pub fn get_subscene_opt(&self, subscene_num_opt: Option<SceneNum>) -> Option<&Scene> {
        if let Some(subscene_num) = subscene_num_opt {
            self.get_subscene(subscene_num)
        } else {
            None
        }
    }
}

impl<'a> From<&'a dyn FilterTrait> for Scene<'a> {
    fn from(ft: &'a dyn FilterTrait) -> Scene<'a> {
        Scene { name: "Single patch", patch: ft, init: &Discard(), exit: &Discard(), subscenes: &[] }
    }
}