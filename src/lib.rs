
mod errors;
mod logger;
mod alloc;
pub mod consts;

#[macro_export]
macro_rules! cfg_rsmemchk_not_inline {
	($($item:item)*) => {
		$(
			#[cfg(feature="rsmemchk_mode")]
			#[inline(never)]
			$item

			#[cfg(not(feature="rsmemchk_mode"))]
			$item
			)*
	}
}


pub use alloc::{StackCallAllocEx,MemoryInfo,MemoryMap,protect_str};
pub use consts::{MEM_READ,MEM_WRITE,MEM_EXEC};
