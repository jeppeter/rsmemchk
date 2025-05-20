
use std::mem::ManuallyDrop;
use std::mem::{null_ptr,null_mut,size_of};

#[cfg(target_os = "windows")]
include!("alloc_windows.rs");

#[cfg(target_os = "linux")]
include!("alloc_linux.rs");

#[repr(C)]
struct MemoryList {
	realptr :*const libc::c_void,
	alignptr :*const libc::c_void,
	next :*const MemoryList,
	callstack :*const libc::c_void,
	callsize :usize,
}

impl MemoryList {
	fn free_mem(ptr :*const MemoryList) {
		if ptr != std::mem::null_ptr() {
			MemoryList::free_mem(ptr.next);
			ptr.next = null_ptr();
			if ptr.callstack != null_ptr() {
				libc::free(ptr.callstack);
				ptr.callstack = null_ptr();
			}
			ptr.realptr = null_ptr();
			ptr.alignptr = null_ptr();
			libc::free(ptr);
		}
		return;
	}

	fn new(stck :&[*const libc::c_void]) -> *const MemoryList {
		let retv :*mut MemoryList;
		retv = libc::malloc(size_of<MemoryList>());
		if retv == std::mem::null_mut() {
			return retv;
		}
		libc::memset(retv,0,size_of<MemoryList>());
		retv.realptr = std::mem::null_ptr();
		retv.alignptr = std::mem::null_ptr();
		retv.next = std::mem::null_ptr();
		retv.callsize = stck.len();
		retv.callstack = libc::malloc(std::mem::size_of<*const libc::c_void>() * retv.callsize);
		if retv.callstack == std::mem::null_ptr() {
			MemoryList::free_mem(retv);
			return std::mem::null_ptr();
		}
		libc::memcpy(retv.callstack, stck.as_ptr(),std::mem::size_of<*const libc::c_void>() *retv.callsize);
		return retv;
	}

	fn take_next(&mut self) -> *const MemoryList {
		let retv = self.next;
		self.next = std::mem::null_ptr();
		return retv;
	}

	fn set_align(&mut self, realptr :*const libc::c_void,align :u32) {
		let mut addr :u64 = realptr as u64;
		self.realptr = realptr;
		addr += (align - 1) as u64;
		addr &= ~((align - 1) as u64);
		self.alignptr = addr as *const libc::c_void;
		return;
	}

	fn set_next(&mut self,other :&MemoryList) -> *const MemoryList {
		let retv = self.next;
		self.next = other as *const MemoryList;
		return retv;
	}
}

#[repr(C)]
pub struct StackCallAlloc {	
	lock : *AllocLock,
	memlist :**mut MemoryList,
	memsize :usize,
}

impl StackCallAlloc {
	fn free_mem(ptr :*const StackCallAlloc) {
		if ptr != std::mem::null_ptr() {
			AllocLock::free_mem(ptr.lock);
			ptr.lock = std::mem::null_ptr();			
		}


	}

	fn new(stacksize :usize) -> *const StackCallAlloc {
		let retv = libc::malloc(std::mem::size_of<StackCallAlloc>());
		if retv == std::mem::null_ptr() {
			return retv;
		}

		libc::memset(retv, 0, std::mem::size_of<StackCallAlloc>());

		retv.lock = AllocLock::new();
		if retv.lock == std::mem::null_ptr() {

		}
	}
}



unsafe impl GlobalAlloc for StackCallAlloc {

}