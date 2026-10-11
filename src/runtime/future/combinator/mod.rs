use crate::driver::Extra;

#[non_exhaustive]
#[derive(Default)]
pub(crate) struct Ext {
    personality: Option<u16>,
}

impl Ext {
    pub fn to_owned(&self) -> Self {
        Self {
            personality: self.personality,
        }
    }
}

impl Ext {
    pub fn set_extra(&self, extra: &mut Extra) -> bool {
        let mut changed = false;
        if let Some(personality) = self.personality {
            extra.set_personality(personality);
            changed = true;
        }
        changed
    }
}
