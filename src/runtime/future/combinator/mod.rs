use crate::driver::Extra;
use core::marker::PhantomData;

#[non_exhaustive]
#[derive(Default)]
pub(crate) struct Ext<'a> {
    personality: Option<u16>,
    _cancel: PhantomData<&'a ()>,
}

impl<'a> Ext<'a> {
    pub fn to_owned(&self) -> Ext<'static> {
        Ext {
            personality: self.personality,
            _cancel: PhantomData,
        }
    }
}

impl<'a> Ext<'a> {
    pub fn set_extra(&self, extra: &mut Extra) -> bool {
        let mut changed = false;
        if let Some(personality) = self.personality {
            extra.set_personality(personality);
            changed = true;
        }
        changed
    }
}
