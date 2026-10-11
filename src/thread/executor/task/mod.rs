use std::{
    array,
    mem::{ManuallyDrop, MaybeUninit, offset_of},
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    ptr::{self, NonNull, drop_in_place},
    sync::atomic::{AtomicPtr, Ordering::*},
    task::{Context, Poll, Waker, ready},
};

use super::{
    PanicResult, SendWrapper, Shared, UnsafeCell,
    console::{SpawnMeta, TaskSpan, WakerOp},
    queue::TaskId,
    task::{
        local::Local,
        remote::Remote,
        state::{State, Strong},
    },
    util::{panic_guard, transpose},
};

mod local;
mod remote;
mod state;

#[repr(transparent)]
pub(crate) struct Task(NonNull<Header>);

#[repr(C)]
struct TaskAlloc<F: Future> {
    header: Header,
    future: UnsafeCell<FutureState<F>>,
}

struct Header {
    id: TaskId,
    state: State,
    vtable: &'static TaskVtable,
    tracker: ManuallyDrop<SendWrapper<()>>,
    shared: AtomicPtr<Shared>,
    waker: UnsafeCell<MaybeUninit<Waker>>,
    /// Zero-sized unless the `console` feature is enabled.
    span: TaskSpan,
}

union FutureState<F: Future> {
    future: ManuallyDrop<F>,
    result: ManuallyDrop<PanicResult<F::Output>>,
}

struct TaskVtable {
    dealloc: unsafe fn(NonNull<Header>),
    run_future: unsafe fn(NonNull<Header>, &mut Context<'_>) -> Poll<()>,
    take_result: unsafe fn(NonNull<Header>, NonNull<()>),
    drop_future: unsafe fn(NonNull<Header>, bool),
}

impl<F: Future + 'static> TaskAlloc<F> {
    const FUT_OFFSET: usize = offset_of!(Self, future);
    const VTABLE: &'static TaskVtable = &TaskVtable {
        dealloc: Self::dealloc,
        run_future: Self::run_future,
        take_result: Self::take_result,
        drop_future: Self::drop_future,
    };

    fn future_cell(header: NonNull<Header>) -> &'static UnsafeCell<FutureState<F>> {
        unsafe {
            &*header
                .byte_add(Self::FUT_OFFSET)
                .cast::<UnsafeCell<FutureState<F>>>()
                .as_ptr()
        }
    }

    unsafe fn run_future(header: NonNull<Header>, cx: &mut Context<'_>) -> Poll<()> {
        let future_cell = Self::future_cell(header);

        let res = ready!(future_cell.with_mut(|fut_ptr| {
            let fut = unsafe { Pin::new_unchecked(&mut *(*fut_ptr).future) };
            transpose(catch_unwind(AssertUnwindSafe(|| fut.poll(cx))))
        }));

        future_cell.with_mut(|fut_ptr| {
            panic_guard!();
            unsafe { drop_in_place(fut_ptr as *mut F) };
            let new_state = FutureState {
                result: ManuallyDrop::new(res),
            };
            unsafe { ptr::write(fut_ptr, new_state) };
        });

        Poll::Ready(())
    }

    unsafe fn take_result(header: NonNull<Header>, target: NonNull<()>) {
        let future_cell = Self::future_cell(header);

        future_cell.with(|fut_ptr| {
            let fut_ptr = fut_ptr as *const PanicResult<F::Output>;
            unsafe { std::ptr::copy_nonoverlapping(fut_ptr, target.as_ptr().cast(), 1) };
        });
    }

    unsafe fn drop_future(header: NonNull<Header>, has_result: bool) {
        if ::std::thread::panicking() {
            return;
        }

        let future_cell = Self::future_cell(header);

        future_cell.with_mut(|fut_ptr| {
            super::panic_guard!();

            if has_result {
                unsafe { drop_in_place::<PanicResult<F::Output>>(fut_ptr as _) };
            } else {
                unsafe { drop_in_place::<F>(fut_ptr as _) };
            }
        });
    }

    unsafe fn dealloc(header: NonNull<Header>) {
        drop(unsafe { Box::from_raw(header.as_ptr().cast::<TaskAlloc<F>>()) });
    }
}

impl Task {
    pub fn new<F: Future + 'static, const N: usize>(
        id: TaskId,
        shared: NonNull<Shared>,
        tracker: SendWrapper<()>,
        future: F,
        meta: SpawnMeta,
    ) -> [Task; N] {
        let alloc = Box::new(TaskAlloc {
            header: Header {
                id,
                state: State::new::<N>(),
                vtable: TaskAlloc::<F>::VTABLE,
                tracker: ManuallyDrop::new(tracker),
                shared: AtomicPtr::new(shared.as_ptr()),
                waker: UnsafeCell::new(MaybeUninit::uninit()),
                span: TaskSpan::new::<F>(meta),
            },
            future: UnsafeCell::new(FutureState {
                future: ManuallyDrop::new(future),
            }),
        });

        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(alloc) as _) };

        array::from_fn(|_| Task(ptr))
    }

    pub unsafe fn from_raw(ptr: *const ()) -> Self {
        Self(NonNull::new(ptr as *mut () as _).expect("Invalid pointer"))
    }

    pub fn as_raw(&self) -> *const () {
        self.0.as_ptr() as _
    }

    pub unsafe fn increment_count(ptr: *const ()) {
        unsafe { &*(ptr as *const Header) }.state.inc();
    }

    #[inline(always)]
    pub(crate) unsafe fn record_waker_op(ptr: *const (), op: WakerOp) {
        unsafe { &*(ptr as *const Header) }.span.waker_op(op);
    }

    pub fn schedule(&self) {
        match self.view() {
            Ok(local) => local.schedule(),
            Err(remote) => remote.schedule(),
        }
    }

    pub fn cancel(&self, drop_result: bool) {
        let header = self.header();

        self.schedule();
        let state = header.state.set_cancelled();
        if drop_result && state.has_result() {
            header.state.set_has_result::<Strong, false>();
            unsafe { (header.vtable.drop_future)(self.0, true) }
        }
    }

    pub unsafe fn run(&self) -> Poll<()> {
        let header = self.header();

        let state = header.state.unschedule();
        if state.is_cancelled() {
            return Poll::Ready(());
        }

        let _entered = header.span.enter();

        self.with_waker(|waker| {
            let ctx = &mut Context::from_waker(waker);
            let res = unsafe { (header.vtable.run_future)(self.0, ctx) };
            if res.is_ready() {
                let state = header.state.finish_running();

                if state.has_waker() && !state.is_setting_waker() {
                    header
                        .waker
                        .with_mut(|ptr| unsafe { (*ptr).assume_init_ref() }.wake_by_ref());
                }
            }

            res
        })
    }

    pub unsafe fn poll<T>(&self, cx: &mut Context<'_>) -> Poll<Option<PanicResult<T>>> {
        match self.view() {
            Ok(local) => unsafe { local.poll(cx) },
            Err(remote) => unsafe { remote.poll(cx) },
        }
    }

    pub unsafe fn drop(&self) {
        let header = self.header();
        debug_assert!(
            header.tracker.valid(),
            "drop should only be called by Executor"
        );
        let state = header.state.set_dropped();

        header.shared.store(ptr::null_mut(), Release);

        if ::std::thread::panicking() {
            return;
        }

        if !state.is_completed() {
            unsafe { (header.vtable.drop_future)(self.0, false) };
        }

        if state.has_waker() && !state.is_setting_waker() {
            super::panic_guard!();

            header
                .waker
                .with_mut(|ptr| unsafe { drop_in_place(ptr.cast::<Waker>()) });
        }
    }

    pub(crate) fn is_finished(&self) -> bool {
        let state = self.header().state.load::<Strong>();
        state.is_completed() || state.is_cancelled()
    }

    pub(crate) fn wait_for_scheduling(&self) {
        let header = self.header();

        while header.state.load::<Strong>().is_scheduling() {
            std::hint::spin_loop();
        }
    }

    fn header(&self) -> &Header {
        unsafe { self.0.as_ref() }
    }

    #[inline(always)]
    fn view(&self) -> Result<Local<'_>, Remote<'_>> {
        if self.header().tracker.valid() {
            Ok(unsafe { Local::new(self.0) })
        } else {
            Err(Remote::new(self.0))
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        let header = self.header();
        let state = header.state.dec();
        if state.count() > 1 {
            return;
        };

        if std::thread::panicking() {
            unsafe { (header.vtable.dealloc)(self.0) }
            return;
        }

        super::panic_guard!();

        debug_assert!(state.is_completed() | state.is_cancelled());
        debug_assert!(!state.is_setting_waker());

        if state.has_result() {
            unsafe { (header.vtable.drop_future)(self.0, true) };
        }

        if state.has_waker() {
            header
                .waker
                .with_mut(|ptr| unsafe { drop_in_place(ptr.cast::<Waker>()) });
        }

        unsafe { (header.vtable.dealloc)(self.0) }
    }
}
