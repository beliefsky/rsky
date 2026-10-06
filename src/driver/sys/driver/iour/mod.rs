use std::{
    io,
    marker::PhantomData,
    mem::ManuallyDrop,
    os::fd::{AsRawFd, RawFd},
    task::{Poll, Waker},
    time::Duration,
};

mod notify;
use notify::Notifier;

mod op;
pub use op::*;
use slotmap::{DefaultKey, SlotMap};

cfg_select! {
    feature = "io-uring-cqe32" => {
        use io_uring::cqueue::Entry32 as CEntry;
    }
    _ => {
        use io_uring::cqueue::Entry as CEntry;
    }
}

cfg_select! {
    feature = "io-uring-sqe128" => {
        use io_uring::squeue::Entry128 as SEntry;
    }
    _ => {
        use io_uring::squeue::Entry as SEntry;
    }
}
use crate::driver::{ProactorBuilder, key::ErasedKey, sys::extra::IourExtra};
use io_uring::{
    EnterFlags, IoUring,
    types::{SubmitArgs, Timespec},
};

struct DriverFlags(u8);

impl DriverFlags {
    const NEED_PUSH_NOTIFIER: u8 = 1 << 0;
    const NO_IOWAIT: u8 = 1 << 1;

    fn new(value: u8) -> Self {
        DriverFlags(value)
    }

    fn set(&mut self, other: u8, value: bool) {
        if value {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }
    fn insert(&mut self, other: u8) {
        self.0 |= other;
    }
    fn remove(&mut self, other: u8) {
        self.0 &= !other;
    }

    fn contains(&self, other: u8) -> bool {
        (self.0 & other) != 0
    }
}

pub(crate) struct Driver {
    inner: ManuallyDrop<IoUring<SEntry, CEntry>>,
    notifier: Notifier,
    flags: DriverFlags,
    in_flight: SlotMap<DefaultKey, usize>,
    _p: PhantomData<ErasedKey>,
}

impl Driver {
    const IOUR_CANCEL: u64 = u64::MAX;
    const IOUR_NOTIFY: u64 = u64::MAX - 1;

    pub fn new(builder: &ProactorBuilder) -> io::Result<Self> {
        let notifier = Notifier::new()?;
        let mut iour_builder = IoUring::builder();
        if let Some(sqpoll_idle) = builder.sqpoll_idle {
            iour_builder.setup_sqpoll(sqpoll_idle.as_millis() as _);
            if let Some(cpu) = builder.sqpoll_cpu {
                iour_builder.setup_sqpoll_cpu(cpu);
            }
        }
        if builder.single_issuer {
            iour_builder.setup_single_issuer();
            if builder.defer_taskrun {
                iour_builder.setup_defer_taskrun();
            }
        }
        if builder.coop_taskrun {
            iour_builder.setup_coop_taskrun();
        }
        if builder.taskrun_flag {
            iour_builder.setup_taskrun_flag();
        }
        if let Some(cqsize) = builder.cqsize {
            iour_builder.setup_cqsize(cqsize);
        }
        iour_builder.dontfork();

        let inner = iour_builder.build(builder.capacity)?;

        let mut flags = DriverFlags::new(DriverFlags::NEED_PUSH_NOTIFIER);
        flags.set(
            DriverFlags::NO_IOWAIT,
            builder.sqpoll_idle.is_none() && inner.params().is_feature_no_iowait(),
        );

        Ok(Self {
            inner: ManuallyDrop::new(inner),
            notifier,
            flags,
            in_flight: SlotMap::new(),
            _p: PhantomData,
        })
    }

    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        // if self.poll_blocking() {
        //     return Ok(());
        // }
        let need_wait = !self.notifier.reset();

        if self.flags.contains(DriverFlags::NEED_PUSH_NOTIFIER) {
            println!("help me，我要加入事件");
            self.flags.remove(DriverFlags::NEED_PUSH_NOTIFIER);
        }

        self.submit_auto(timeout, need_wait)?;

        self.notifier.set_awake();
        self.poll_entries();
        self.notifier.set_awake();

        Ok(())
    }

    pub(in crate::driver::sys) fn default_extra(&self) -> IourExtra {
        IourExtra::new()
    }

    pub fn push(&mut self, key: ErasedKey) -> Poll<io::Result<usize>> {
        let op_entry = key.borrow().create_entry();
        match op_entry {
            OpEntry::Submission(entry) => {
                println!("OpEntry::Submission");
                self.push_raw_with_key(entry.into(), key)?;
            }
            #[cfg(feature = "io-uring-sqe128")]
            OpEntry::Submission128(entry) => {}
            OpEntry::Blocking => {
                println!("OpEntry::Blocking");
            }
        }

        /*
        let mut op_entry = key.borrow().create_entry::<false>();
        let mut has_fallbacked = false;
        loop {
            match op_entry {
                OpEntry::Submission(entry) => {
                    if is_op_supported(entry.get_opcode() as _) {
                        #[allow(clippy::useless_conversion)]
                        self.push_raw_with_key(entry.into(), key)?;
                    } else if !has_fallbacked {
                        op_entry = key.borrow().create_entry::<true>();
                        has_fallbacked = true;
                        continue;
                    } else {
                        self.push_blocking(key);
                    }
                }
                #[cfg(feature = "io-uring-sqe128")]
                OpEntry::Submission128(entry) => {
                    if is_op_supported(entry.get_opcode() as _) {
                        self.push_raw_with_key(entry, key)?;
                    } else if !has_fallbacked {
                        op_entry = key.borrow().create_entry::<true>();
                        has_fallbacked = true;
                        continue;
                    } else {
                        self.push_blocking(key);
                    }
                }
                OpEntry::Blocking => self.push_blocking(key),
            }
            break;
        }
        */
        Poll::Pending
    }

    pub fn waker(&self) -> Waker {
        self.notifier.waker()
    }

    fn submit_auto(&mut self, timeout: Option<Duration>, need_wait: bool) -> io::Result<()> {
        let want_sqe = if !need_wait || self.inner.submission().taskrun() {
            0
        } else {
            1
        };
        let can_block = want_sqe > 0 && timeout != Some(Duration::ZERO);
        let res = if self.flags.contains(DriverFlags::NO_IOWAIT) && can_block {
            self.submit_and_wait_no_iowait(want_sqe, timeout)
        } else {
            self.submit_and_wait(want_sqe, timeout)
        };
        match res {
            Ok(_) => {
                if want_sqe > 0 && self.inner.completion().is_empty() {
                    Err(io::ErrorKind::TimedOut.into())
                } else {
                    Ok(())
                }
            }
            Err(e) => match e.raw_os_error() {
                // Some(libc::ETIME) => Err(io::ErrorKind::TimedOut.into()),
                // Some(libc::EBUSY) | Some(libc::EAGAIN) => Err(io::ErrorKind::Interrupted.into()),
                _ => Err(e),
            },
        }
    }

    fn submit_and_wait(&self, want_sqe: usize, timeout: Option<Duration>) -> io::Result<usize> {
        if let Some(duration) = timeout {
            let timespec = timespec(duration);
            let args = SubmitArgs::new().timespec(&timespec);
            self.inner.submitter().submit_with_args(want_sqe, &args)
        } else {
            self.inner.submit_and_wait(want_sqe)
        }
    }

    fn submit_and_wait_no_iowait(
        &mut self,
        want_sqe: usize,
        timeout: Option<Duration>,
    ) -> io::Result<usize> {
        // Publish the SQ tail and read how many staged SQEs to submit this
        // call.
        let to_submit = self.inner.submission().len() as u32;
        let submitter = self.inner.submitter();
        if let Some(duration) = timeout {
            let timespec = timespec(duration);
            let args = SubmitArgs::new().timespec(&timespec);
            let flags = EnterFlags::EXT_ARG | EnterFlags::GETEVENTS | EnterFlags::NO_IOWAIT;
            // SAFETY: `args` outlives the call; the SQ is synced and holds
            // `to_submit` valid SQEs.
            unsafe { submitter.enter(to_submit, want_sqe as u32, flags.bits(), Some(&args)) }
        } else {
            #[repr(C)] //libc::sigset_t
            pub struct sigset_t {
                #[cfg(target_pointer_width = "32")]
                __val: [u32; 32],
                #[cfg(target_pointer_width = "64")]
                __val: [u64; 16],
            }

            let flags = EnterFlags::GETEVENTS | EnterFlags::NO_IOWAIT;
            // SAFETY: the SQ is synced and holds `to_submit` valid SQEs; no arg
            // payload is referenced.
            unsafe { submitter.enter::<sigset_t>(to_submit, want_sqe as u32, flags.bits(), None) }
        }
    }

    fn poll_entries(&mut self) -> bool {
        let cqueue = self.inner.completion();
        let has_entry = !cqueue.is_empty();
        for entry in cqueue {
            match entry.user_data() {
                Self::IOUR_CANCEL => {}
                Self::IOUR_NOTIFY => {
                    let flags = entry.flags();
                    if !io_uring::cqueue::more(flags) {
                        self.flags.insert(DriverFlags::NEED_PUSH_NOTIFIER);
                    }
                    println!("iouring NOTIFY");
                }
                key => {
                    println!("event data: {}", key);
                    let flags = entry.flags();
                    if io_uring::cqueue::more(flags) {
                        println!("event flag: more");
                    } else {
                        let entry = create_entry(entry);
                        Self::remove_in_flight(&mut self.in_flight, &entry.key);
                        entry.notify();
                    }
                }
            }
        }
        has_entry
    }

    fn push_raw_with_key(&mut self, entry: SEntry, key: ErasedKey) -> io::Result<()> {
        let user_data = key.as_raw();
        let entry = entry.user_data(user_data as _);
        self.push_raw(entry)?; // if push failed, do not leak the key. Drop it upon return.

        let slot = self.in_flight.insert(user_data);
        key.borrow().extra_mut().set_in_flight(slot);
        key.into_raw(); // 对象内存托管在in_flight中以指针地址存在，需要手动释放
        Ok(())
    }

    fn push_raw(&mut self, entry: SEntry) -> io::Result<()> {
        loop {
            let mut squeue = self.inner.submission();
            match unsafe { squeue.push(&entry) } {
                Ok(()) => {
                    squeue.sync();
                    break Ok(());
                }
                // 列队满了会失败，需提交后再加入列队
                Err(_) => {
                    drop(squeue);
                    match self.submit_auto(Some(Duration::ZERO), true) {
                        Ok(()) => {}
                        Err(e)
                            if matches!(
                                e.kind(),
                                io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                            ) => {}
                        Err(e) => return Err(e),
                    }
                    // 提交后直接处理完成的列队
                    if self.poll_entries() {
                        // 唤醒去处理后续任务
                        self.notifier.waker().wake();
                    }
                }
            }
        }
    }

    fn remove_in_flight(in_flight: &mut SlotMap<DefaultKey, usize>, key: &ErasedKey) {
        let Some(slot) = key.borrow().extra_mut().take_in_flight() else {
            return;
        };
        let removed = in_flight.remove(slot);
        debug_assert_eq!(removed, Some(key.as_raw()));
    }
}

impl AsRawFd for Driver {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.inner) };

        //手动释放指针数据
        for (_, user_data) in self.in_flight.drain() {
            drop(unsafe { ErasedKey::from_raw(user_data) });
        }
    }
}

pub(crate) struct Entry {
    key: ErasedKey,
    result: io::Result<usize>,
    flags: u32,
}

impl Entry {
    pub(crate) fn new(key: ErasedKey, result: io::Result<usize>) -> Self {
        Self {
            key,
            result,
            flags: 0,
        }
    }

    pub(crate) fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    pub fn notify(self) {
        self.key.borrow().extra_mut().set_flags(self.flags);
        self.key.set_result(self.result);
    }
}

fn create_entry(cq_entry: CEntry) -> Entry {
    let result = cq_entry.result();
    let result = create_result(result);
    let key = unsafe { ErasedKey::from_raw(cq_entry.user_data() as _) };
    let mut entry = Entry::new(key, result);
    entry.set_flags(cq_entry.flags());

    entry
}

fn create_result(result: i32) -> io::Result<usize> {
    const ERR_ENOBUFS: i32 = -105;

    if result < 0 {
        // ENOBUFS indicates the io_uring buffer pool has no available buffer.
        if result == ERR_ENOBUFS {
            Err(io::Error::new(
                io::ErrorKind::ResourceBusy,
                "buffer ring has no available buffer",
            ))
        } else {
            Err(io::Error::from_raw_os_error(-result))
        }
    } else {
        Ok(result as _)
    }
}

fn timespec(duration: std::time::Duration) -> Timespec {
    Timespec::new()
        .sec(duration.as_secs())
        .nsec(duration.subsec_nanos())
}
