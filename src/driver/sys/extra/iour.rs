use io_uring::squeue::Flags;
use slotmap::DefaultKey;

pub(in crate::driver::sys) struct Extra {
    sqe_flags: Flags,
    cqe_flags: u32,
    personality: Option<u16>,
    /// Slot of this op inside the `in_flight` map of the `io_uring` driver,
    /// set while the op is in flight.
    in_flight: Option<DefaultKey>,
}

pub(in crate::driver::sys) use Extra as IourExtra;

impl Extra {
    pub fn new() -> Self {
        Self {
            sqe_flags: Flags::empty(),
            cqe_flags: 0,
            personality: None,
            in_flight: None,
        }
    }

    pub fn set_in_flight(&mut self, slot: DefaultKey) {
        debug_assert!(self.in_flight.is_none(), "op is already in flight");
        self.in_flight = Some(slot);
    }

    pub fn take_in_flight(&mut self) -> Option<DefaultKey> {
        self.in_flight.take()
    }

    pub fn set_personality(&mut self, personality: u16) {
        self.personality = Some(personality);
    }

    pub fn set_link(&mut self) {
        self.sqe_flags |= Flags::IO_LINK;
    }

    pub fn set_hardlink(&mut self) {
        self.sqe_flags |= Flags::IO_HARDLINK;
    }

    pub fn set_drain(&mut self) {
        self.sqe_flags |= Flags::IO_DRAIN;
    }

    pub fn get_personality(&self) -> Option<u16> {
        self.personality
    }

    pub fn get_sqe_flags(&self) -> Flags {
        self.sqe_flags
    }

    pub fn buffer_id(&self) -> Option<u16> {
        io_uring::cqueue::buffer_select(self.cqe_flags)
    }

    pub fn sock_nonempty(&self) -> bool {
        io_uring::cqueue::sock_nonempty(self.cqe_flags)
    }

    pub fn is_notification(&self) -> bool {
        io_uring::cqueue::notif(self.cqe_flags)
    }
}

impl super::Extra {
    pub(crate) fn set_in_flight(&mut self, slot: DefaultKey) {
        self.0.set_in_flight(slot);
    }

    pub fn take_in_flight(&mut self) -> Option<DefaultKey> {
        self.0.take_in_flight()
    }

    pub(crate) fn get_personality(&self) -> Option<u16> {
        self.0.get_personality()
    }
    pub(crate) fn set_personality(&mut self, personality: u16) {
        self.0.set_personality(personality);
    }

    pub(crate) fn get_sqe_flags(&self) -> Flags {
        self.0.get_sqe_flags()
    }

    pub(crate) fn set_flags(&mut self, flag: u32) {
        self.0.cqe_flags = flag
    }
}
