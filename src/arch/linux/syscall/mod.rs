
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
pub(in crate::arch::linux) mod asm;

#[cfg(any(
    target_arch = "x86_64",
))]
pub(in crate::arch::linux) use self::asm as choose;


macro_rules! syscall_readonly {
    ($nr:ident) => {
        $crate::arch::linux::choose::syscall0_readonly($crate::backend::reg::nr(
            linux_raw_sys::general::$nr,
        ))
    };

    ($nr:ident, $a0:expr) => {
        $crate::arch::linux::choose::syscall1_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
        )
    };

    ($nr:ident, $a0:expr, $a1:expr) => {
        $crate::arch::linux::choose::syscall2_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
            $a1.into(),
        )
    };

    ($nr:ident, $a0:expr, $a1:expr, $a2:expr) => {
        $crate::arch::linux::choose::syscall3_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
            $a1.into(),
            $a2.into(),
        )
    };

    ($nr:ident, $a0:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::arch::linux::choose::syscall4_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
            $a1.into(),
            $a2.into(),
            $a3.into(),
        )
    };

    ($nr:ident, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        $ccrate::arch::linux::choose::syscall5_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
            $a1.into(),
            $a2.into(),
            $a3.into(),
            $a4.into(),
        )
    };

    ($nr:ident, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {
        $ccrate::arch::linux::choose::syscall6_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
            $a1.into(),
            $a2.into(),
            $a3.into(),
            $a4.into(),
            $a5.into(),
        )
    };

    ($nr:ident, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {
        $crate::arch::linux::choose::syscall7_readonly(
            $crate::backend::reg::nr(linux_raw_sys::general::$nr),
            $a0.into(),
            $a1.into(),
            $a2.into(),
            $a3.into(),
            $a4.into(),
            $a5.into(),
            $a6.into(),
        )
    };
}