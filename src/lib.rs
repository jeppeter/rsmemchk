
mod errors;
mod logger;
mod alloc;

#[macro_export]
macro_rules! cfg_rsmalloc_not_inline {
	($($item:item)*) => {
		$(
			#[cfg(feature="rsmalloc_mode")]
			#[inline(never)]
			$item

			#[cfg(not(feature="rsmalloc_mode"))]
			$item
			)*
	}
}


pub use alloc::{StackCallAllocEx,MemoryInfo,MemoryMap};