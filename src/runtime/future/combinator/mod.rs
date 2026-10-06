use std::borrow::Cow;

use crate::{driver::Extra, runtime::CancelToken};

#[non_exhaustive]
#[derive(Default)]
pub(crate) struct Ext<'a> {
    personality: Option<u16>,
    cancel: Option<Cow<'a, CancelToken>>,
}

impl<'a> Ext<'a> {
    pub fn to_owned(&self) -> Ext<'static> {
        Ext {
            personality: self.personality,
            cancel: self
                .cancel
                .as_ref()
                .map(|x| Cow::Owned(x.clone().into_owned())),
        }
    }
}

impl<'a> Ext<'a> {
    pub fn get_cancel(&self) -> Option<&CancelToken> {
        self.cancel.as_deref()
    }

    pub fn set_extra(&self, extra: &mut Extra) -> bool {
        let mut changed = false;
        if let Some(personality) = self.personality {
            extra.set_personality(personality);
            changed = true;
        }
        changed
    }
}
