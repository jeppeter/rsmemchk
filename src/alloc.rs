
use std::alloc::{GlobalAlloc,Layout};
use std::mem::{size_of};
use std::ptr::{null_mut};

#[cfg(target_os = "windows")]
include!("alloc_windows.rs");

#[cfg(target_os = "linux")]
include!("alloc_linux.rs");


#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _write_str(s :&str) {
	let _ptr :*const u8 = s.as_bytes().as_ptr();
	libc::write(2,_ptr as *const libc::c_void,s.len() as u32);
}

#[allow(unused_mut)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _write_val(val :u64, ishex :bool) {
	let mut cbuf :[u8;32] = [0;32];
	let mut clen :usize = 0;
	let mut obuf :[u8;32] = [0;32];
	let mut cval :u64 = val;

	if ishex {
		while cval > 0 {
			let curval :u8 = (cval & 0xf) as u8;
			if  curval <= 9 {
				cbuf[clen] = b'0' + curval;
			} else {
				cbuf[clen] = b'a' + (curval - 10);
			}
			clen += 1;
			cval >>= 4;
		}

		cbuf[clen] = b'x';
		clen += 1;
		cbuf[clen] = b'0';
		clen += 1
	} else {
		while cval > 0 {
			let curval :u8 = (cval % 10) as u8;
			cbuf[clen] = b'0' + curval;
			clen += 1;
			cval = cval / 10;
		}
	}

	if clen == 0 {
		obuf[0] = b'0';
		clen += 1;
	} else {
		for i in 0..clen {
			obuf[i] = cbuf[clen - i-1];
		}
	}
	let _ptr :*const u8 = obuf.as_ptr();
	libc::write(2,_ptr as *const libc::c_void,clen as u32);
	return;
}


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

	unsafe fn new(stck :*mut *const libc::c_void, stksize :usize) -> *mut MemoryList {
		let retv :*mut MemoryList;
		retv = libc::malloc(size_of::<MemoryList>()) as *mut MemoryList;
		if retv == null_mut() {
			return retv;
		}
		libc::memset(retv as *mut libc::c_void,0,size_of::<MemoryList>());
		(*retv).realptr = null_mut();
		(*retv).alignptr =null_mut();
		(*retv).next = null_mut();
		(*retv).callsize = stksize;
		(*retv).callstack = libc::malloc(size_of::<*const libc::c_void>() * (*retv).callsize) as *mut *const libc::c_void ;
		if (*retv).callstack == null_mut() {
			MemoryList::free_mem(retv);
			return null_mut();
		}
		libc::memcpy((*retv).callstack as *mut libc::c_void, stck as *const libc::c_void,size_of::<*const libc::c_void>() * (*retv).callsize);
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

//const TRACE_LEVEL :i32 = 40;
const DEBUG_LEVEL :i32 = 30;
//const INFO_LEVEL :i32 = 20;
//const WARN_LEVEL :i32 = 10;
const ERROR_LEVEL:i32 = 0;


#[repr(C)]
pub struct StackCallAlloc {	
	lock : *mut AllocLock,
	memlist :*mut *mut MemoryList,
	memsize :usize,
	loglvl : i32,
}

const BACK_MEM_SIZE :usize = 4;

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
		return;
	}

	fn _hash_value(&self, val :u64) -> usize {
		return (val % self.memsize as u64) as usize;
	}

	fn _debug_write_str(&self,s :&str) {
		if self.loglvl >= DEBUG_LEVEL {
			unsafe {
				_write_str(s);	
			}
			
		}		
	}

	fn _debug_write_val(&self, val :u64 , ishex:bool) {
		if self.loglvl >= DEBUG_LEVEL {
			unsafe {
				_write_val(val,ishex);	
			}			
		}
	}

	fn _error_write_str(&self, s:&str) {
		if self.loglvl >= ERROR_LEVEL {
			unsafe {
				_write_str(s);	
			}
			
		}
	}

	fn _error_write_val(&self, val :u64, ishex :bool) {
		if self.loglvl >= ERROR_LEVEL {
			unsafe {
				_write_val(val,ishex);	
			}			
		}
	}

	unsafe fn _dealloc_inner(&self, ptr :*mut u8, _layout :Layout) -> i32{
		let iv :usize = self._hash_value(ptr as u64);
		let meml :*mut MemoryList = (*self.memlist.wrapping_add(iv)) as *mut MemoryList;
		let mut retv :i32 = 0;
		let mut pcur :*mut MemoryList;
		let mut pnext :*mut MemoryList;
		let mut pprev :*mut MemoryList = null_mut();
		if meml != null_mut() {
			/*now to search for the value*/
			pcur = meml;
			pnext = (*pcur).next;
			while pcur != null_mut() {
				if (*pcur).alignptr == ptr {
					retv = 1;
					break;
				}
				pprev = pcur;
				pcur = pnext;
				if pnext != null_mut() {
					pnext = (*pnext).next;
				}
			}

			if pprev != null_mut() {
				(*pprev).next = pnext;
			} else {
				(*self.memlist.wrapping_add(iv)) = pnext;
			}

			if pcur != null_mut() {
				(*pcur).next = null_mut();
				libc::free((*pcur).realptr);
				(*pcur).realptr = null_mut();
				MemoryList::free_mem(pcur);
			}
		}
		return retv;
	}

	unsafe fn _alloc_inner(&self, realptr :*mut libc::c_void, alignptr :*mut u8 ) -> i32 {
		let retv :i32;
		let backs :*mut *mut libc::c_void = libc::malloc(size_of::<*mut libc::c_void>() * BACK_MEM_SIZE) as *mut *mut libc::c_void;
		if backs == null_mut() {
			return -1;
		}

		retv = _get_stack_call(0,backs,BACK_MEM_SIZE);
		if retv < 0 {
			libc::free(backs as *mut libc::c_void);
			return -1;
		}
		self._debug_write_str("[");
		self._debug_write_str(file!());
		self._debug_write_str(":");
		self._debug_write_val(line!() as u64, false);
		self._debug_write_str("]:");
		self._debug_write_str("alignptr [");
		self._debug_write_val(alignptr as u64, true);
		self._debug_write_str("] realptr [");
		self._debug_write_val(realptr as u64, true);
		self._debug_write_str("] with backtrace [");
		for i in 0..BACK_MEM_SIZE {
			if i > 0 {
				self._debug_write_str(",");
			}
			self._debug_write_val(*backs.wrapping_add(i) as u64, true);
		}
		self._debug_write_str("]\n");


		let meml :*mut MemoryList = MemoryList::new(backs as *mut *const libc::c_void,retv as usize);
		libc::free(backs as *mut libc::c_void);		
		if meml == null_mut() {
			return -1;
		}


		(*meml).alignptr = alignptr;
		(*meml).realptr = realptr;


		let hashval = self._hash_value( alignptr as u64);
		let prev :*mut MemoryList = *self.memlist.wrapping_add(hashval);
		if prev == null_mut() {
			*self.memlist.wrapping_add(hashval) = meml;
		} else {
			(*meml).next = prev;
			*self.memlist.wrapping_add(hashval) = meml;
		}
		return 1;
	}

	pub unsafe fn new(stacksize :usize) -> *mut StackCallAlloc {
		let retv :*mut StackCallAlloc = libc::malloc(size_of::<StackCallAlloc>()) as *mut StackCallAlloc;
		let retstr :*mut libc::c_char;
		if retv == null_mut() {
			return retv;
		}

		libc::memset(retv as *mut libc::c_void, 0, size_of::<StackCallAlloc>());

		(*retv).lock = AllocLock::new();
		if (*retv).lock == null_mut() {
			Self::free_mem(retv);
			return null_mut();
		}
		(*retv).loglvl = 0;
		retstr = libc::getenv("RSMALLOC_LOGLEVEL".as_bytes().as_ptr() as *const i8);
		if retstr != null_mut() {
			(*retv).loglvl = libc::atoi(retstr);
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


#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for StackCallAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    	let ptr :*mut libc::c_void ;
    	let retptr :*mut u8;
    	let mut addr :u64;
    	let mut allsize :usize;
    	let retv :i32;
    	allsize = layout.size();
    	if layout.align() > 0 {
    		allsize += layout.align() as usize - 1;	
    	}
    	
    	ptr = libc::malloc(allsize);
    	if ptr == null_mut() {
    		return null_mut();
    	}

    	addr = ptr as u64;
    	if layout.align() > 1 {
	    	addr += layout.align() as u64 - 1;
	    	addr &= !(layout.align() as u64 - 1);
    	}
    	retptr = addr as *mut u8;
    	(*self.lock).lock();
    	retv = self._alloc_inner(ptr,retptr);
    	(*self.lock).unlock();

    	if retv < 0 {
    		/*not insert*/
    		libc::free(ptr);
    		return null_mut();
    	}

    	retptr
    }


    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    	let retv :i32;
    	(*self.lock).lock();
    	retv=  self._dealloc_inner(_ptr,_layout);
    	(*self.lock).unlock();
    	if retv == 0 {
    		libc::free(_ptr as *mut libc::c_void);
    	}
    	return;

    }

}