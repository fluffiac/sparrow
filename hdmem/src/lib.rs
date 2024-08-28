mod mem;
mod util;

use util::builder;

pub use mem::HdMemReader;
pub use win_wrap::Proc;

/// The memory reader type for whole application
type MemReader = Proc;
