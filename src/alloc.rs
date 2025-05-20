
use std::alloc::{GlobalAlloc,Layout};
use std::mem::{size_of};
use std::ptr::{null_mut};

#[cfg(target_os = "windows")]
include!("alloc_windows.rs");

#[cfg(target_os = "linux")]
include!("alloc_linux.rs");

#[repr(C)]
struct MemoryList {
	realptr :*mut libc::c_void,
	alignptr :*mut u8,
	next :*mut MemoryList,
	callstack :*mut *const libc::c_void,
	callsize :usize,
}

#[allow(dead_code)]
#[allow(unsafe_op_in_unsafe_fn)]
impl MemoryList {
	unsafe fn free_mem(ptr :*mut MemoryList) {
		if ptr != null_mut() {
			MemoryList::free_mem((&(*ptr)).next);
			(*ptr).next = null_mut();
			if (*ptr).callstack != null_mut() {
				libc::free((*ptr).callstack as *mut libc::c_void);
				(*ptr).callstack = null_mut();
			}
			(*ptr).realptr = null_mut();
			(*ptr).alignptr = null_mut();
			libc::free(ptr as *mut libc::c_void);
		}
		return;
	}

	unsafe fn new(stck :&[*const libc::c_void]) -> *mut MemoryList {
		let retv :*mut MemoryList;
		retv = libc::malloc(size_of::<MemoryList>()) as *mut MemoryList;
		if retv == null_mut() {
			return retv;
		}
		libc::memset(retv as *mut libc::c_void,0,size_of::<MemoryList>());
		(*retv).realptr = null_mut();
		(*retv).alignptr =null_mut();
		(*retv).next = null_mut();
		(*retv).callsize = stck.len();
		(*retv).callstack = libc::malloc(size_of::<*const libc::c_void>() * (*retv).callsize) as *mut *const libc::c_void ;
		if (*retv).callstack == null_mut() {
			MemoryList::free_mem(retv);
			return null_mut();
		}
		libc::memcpy((*retv).callstack as *mut libc::c_void, stck.as_ptr() as *const libc::c_void,size_of::<*const libc::c_void>() * (*retv).callsize);
		return retv;
	}

	fn take_next(&mut self) -> *const MemoryList {
		let retv = self.next;
		self.next = null_mut();
		return retv;
	}

	fn set_align(&mut self, realptr :*mut libc::c_void,align :u32) {
		let mut addr :u64 = realptr as u64;
		self.realptr = realptr;
		addr += (align - 1) as u64;
		addr &= !((align - 1) as u64);
		self.alignptr = addr as *mut u8;
		return;
	}

	fn set_next(&mut self,other :* mut MemoryList) -> *mut MemoryList {
		let retv = self.next;
		self.next = other;
		return retv;
	}
}

#[repr(C)]
pub struct StackCallAlloc {	
	lock : *mut AllocLock,
	memlist :*mut *mut MemoryList,
	memsize :usize,
}

#[allow(dead_code)]
#[allow(unsafe_op_in_unsafe_fn)]
impl StackCallAlloc {
	pub unsafe fn free_mem(ptr :*mut StackCallAlloc) {
		if ptr != null_mut() {
			let mut i :usize;
			if (*ptr).memlist != null_mut() {
				i = 0;
				while i < (*ptr).memsize {
					let curmem :*mut MemoryList = *((*ptr).memlist.wrapping_add(i));
					if curmem != null_mut() {
						MemoryList::free_mem(curmem);
					}
					*((*ptr).memlist.wrapping_add(i)) = null_mut();
					i += 1;
				}
				libc::free((*ptr).memlist as *mut libc::c_void);
				(*ptr).memlist = null_mut();
			}
			(*ptr).memsize = 0;

			AllocLock::free_mem((&(*ptr)).lock);
			(*ptr).lock = null_mut();
		}


	}

	pub unsafe fn new(stacksize :usize) -> *mut StackCallAlloc {
		let retv :*mut StackCallAlloc = libc::malloc(size_of::<StackCallAlloc>()) as *mut StackCallAlloc;
		if retv == null_mut() {
			return retv;
		}

		libc::memset(retv as *mut libc::c_void, 0, size_of::<StackCallAlloc>());

		(*retv).lock = AllocLock::new();
		if (*retv).lock == null_mut() {
			Self::free_mem(retv);
			return null_mut();
		}
		(*retv).memsize = stacksize;
		(*retv).memlist = libc::malloc(size_of::<*mut MemoryList>() * stacksize) as *mut *mut MemoryList;
		if (*retv).memlist == null_mut() {
			Self::free_mem(retv);
			return null_mut();
		}

		retv
	}
}



unsafe impl GlobalAlloc for StackCallAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
    	let ptr :*mut u8 = null_mut();
    	ptr
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    }

}