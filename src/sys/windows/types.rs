
pub const INVALID_HANDLE_VALUE: HANDLE = -1i32 as _;
pub const NULL_HANDLE_VALUE: HANDLE = 0usize;

pub const TRUE: BOOL = 1i32;
pub const FLASE: BOOL = 0i32;

pub type SOCKET = usize;
pub type HANDLE = usize;
pub type BOOL = i32;


pub const IPPROTO_TCP: i32 = 6i32;
pub const IPPROTO_UDP: i32 = 17i32;

pub const AF_INET: u16 = 2u16;
pub const AF_INET6: u16 = 23u16;

pub const SOCK_STREAM: i32 = 1i32;


pub const FIONBIO: i32 = -2147195266i32;

pub const INVALID_SOCKET: SOCKET = -1i32 as _;

pub const HANDLE_FLAG_INHERIT: u32 =  1u32;
pub const HANDLE_FLAG_PROTECT_FROM_CLOSE: u32 = 2u32;