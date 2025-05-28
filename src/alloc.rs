
use std::alloc::{GlobalAlloc,Layout};
use std::mem::{size_of};
#[allow(unused_imports)]
use std::ptr::{null_mut,null};
use std::error::Error;
use crate::*;
#[allow(unused_imports)]
use crate::logger::*;

pub struct MemoryMap {
	pub startaddr :u64,
	pub endaddr :u64,
	pub mapfile :String,
}

pub struct MemoryInfo {
	pub maps :Vec<MemoryMap>,
}

impl MemoryMap {
	pub fn new() -> Self {
		Self {
			startaddr : 0,
			endaddr :0,
			mapfile : format!(""),
		}
	}
}

impl MemoryInfo {
	pub fn new() -> Self {
		Self {
			maps :vec![],
		}
	}
}

rsmalloc_error_class!{RsAllocError}

#[cfg(target_os = "windows")]
include!("alloc_windows.rs");

#[cfg(target_os = "linux")]
include!("alloc_linux.rs");

const ALLOC_DEFAULT_FD :libc::c_int = 2;

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _write_str(fd :libc::c_int,s :&str) {
	let _ptr :*const u8 = s.as_bytes().as_ptr();
	_write_func(fd,_ptr as *const libc::c_void,s.len() as u32);
}

#[allow(unused_mut)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _write_val(fd: libc::c_int,val :u64, ishex :bool) {
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

		if clen == 0 {
			cbuf[clen] = b'0';
			clen += 1;
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
		if clen == 0 {
			cbuf[clen] = b'0';
			clen += 1;
		}
	}

	for i in 0..clen {
		obuf[i] = cbuf[clen - i-1];
	}
	let _ptr :*const u8 = obuf.as_ptr();

	_write_func(fd,_ptr as *const libc::c_void,clen as u32);
	return;
}


#[repr(C)]
struct MemoryList {
	realptr :*mut libc::c_void,
	alignptr :*mut u8,
	size :usize,
	alignsize :usize,
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
		(*retv).size = 0;
		(*retv).alignsize = 0;
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
}

//const TRACE_LEVEL :i32 = 40;
const DEBUG_LEVEL :i32 = 30;
//const INFO_LEVEL :i32 = 20;
//const WARN_LEVEL :i32 = 10;
const ERROR_LEVEL:i32 = 10;
#[allow(dead_code)]
const FATAL_LEVEL:i32 = 0;


#[repr(C)]
struct StackCallAlloc {	
	lock : *mut AllocLock,
	memlist :*mut *mut MemoryList,
	memsize :usize,
	loglvl : i32,
	fd :libc::c_int,
}

#[repr(C)]
pub struct StackCallAllocEx {
	pub memsize :usize,
}

static mut GLBL_ALLOC :*mut StackCallAlloc = null_mut();

const BACK_MEM_SIZE :usize = 8;

unsafe impl Sync for StackCallAlloc {}
unsafe impl Sync for StackCallAllocEx {}

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

			if (*ptr).fd != ALLOC_DEFAULT_FD && (*ptr).fd >= 0 {
				libc::close((*ptr).fd);
			}
			(*ptr).fd = -1;
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
				_write_str(self.fd,s);	
			}
			
		}		
	}

	fn _debug_write_val(&self, val :u64 , ishex:bool) {
		if self.loglvl >= DEBUG_LEVEL {
			unsafe {
				_write_val(self.fd,val,ishex);	
			}			
		}
	}

	fn _error_write_str(&self, s:&str) {
		if self.loglvl >= ERROR_LEVEL {
			unsafe {
				_write_str(self.fd,s);	
			}
			
		}
	}

	fn _error_write_val(&self, val :u64, ishex :bool) {
		if self.loglvl >= ERROR_LEVEL {
			unsafe {
				_write_val(self.fd,val,ishex);	
			}			
		}
	}

	fn _error_file_line(&self, f :&str ,lineno :u32) {
		if self.loglvl >= ERROR_LEVEL {
			unsafe {
				_write_str(self.fd,"[RSMALLOC]<ERROR>:");
				_write_str(self.fd,"[");
				_write_str(self.fd,f);
				_write_str(self.fd,":");
				_write_val(self.fd,lineno as u64, false);
				_write_str(self.fd,"]:");
			}
		}
	}

	fn _debug_file_line(&self, f :&str ,lineno :u32) {
		if self.loglvl >= DEBUG_LEVEL {
			unsafe {
				_write_str(self.fd,"[RSMALLOC]<DEBUG>:");
				_write_str(self.fd,"[");
				_write_str(self.fd,f);
				_write_str(self.fd,":");
				_write_val(self.fd,lineno as u64, false);
				_write_str(self.fd,"]:");
			}
		}
	}


	unsafe fn _dealloc_inner(&self, ptr :*mut u8, _layout :Layout) -> i32{
		let iv :usize = self._hash_value(ptr as u64);
		let mut jdx :usize;
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

			if retv != 0 {
				if pprev != null_mut() {
					(*pprev).next = pnext;
				} else {
					(*self.memlist.wrapping_add(iv)) = pnext;
				}

				if pcur != null_mut() {
					self._debug_file_line(file!(),line!());
					self._debug_write_str("deallocate: alignptr[");
					self._debug_write_val((*pcur).alignptr as u64,true);
					self._debug_write_str("] realptr[");
					self._debug_write_val((*pcur).realptr as u64, true);
					self._debug_write_str("] with backs [");
					jdx = 0;
					while jdx < (*pcur).callsize {
						let p :*const libc::c_void = *((*pcur).callstack.wrapping_add(jdx));
						if jdx > 0 {
							self._debug_write_str(",");
						}
						self._debug_write_val(p as u64, true);
						jdx += 1;
					}
					self._debug_write_str("]\n");
					(*pcur).next = null_mut();
					libc::free((*pcur).realptr);
					(*pcur).realptr = null_mut();
					MemoryList::free_mem(pcur);
				}				
			}
		}
		return retv;
	}

	unsafe fn _alloc_inner(&self, realptr :*mut libc::c_void, alignptr :*mut u8, layout :&Layout) -> i32 {
		let retv :i32;
		let backs :*mut *mut libc::c_void = libc::malloc(size_of::<*mut libc::c_void>() * BACK_MEM_SIZE) as *mut *mut libc::c_void;
		if backs == null_mut() {
			return -1;
		}

		libc::memset(backs as *mut libc::c_void, 0, size_of::<*mut libc::c_void>() * BACK_MEM_SIZE);
		retv = _get_stack_call(0,backs,BACK_MEM_SIZE);
		if retv < 0 {
			libc::free(backs as *mut libc::c_void);
			return -1;
		}
		self._debug_file_line(file!(),line!());
		self._debug_write_str("allocate:");
		self._debug_write_str("alignptr [");
		self._debug_write_val(alignptr as u64, true);
		self._debug_write_str("] realptr [");
		self._debug_write_val(realptr as u64, true);
		self._debug_write_str("] size [");
		self._debug_write_val(layout.size() as u64, true);
		self._debug_write_str("] align [");
		self._debug_write_val(layout.align() as u64, true);
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
		(*meml).size = layout.size();
		(*meml).alignsize = layout.align();


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

	pub  fn new(stacksize :usize) -> *mut StackCallAlloc {
		unsafe {
			let retv :*mut StackCallAlloc = libc::malloc(size_of::<StackCallAlloc>()) as *mut StackCallAlloc;
			let mut retstr :*mut libc::c_char;
			if retv == null_mut() {
				return retv;
			}

			libc::memset(retv as *mut libc::c_void, 0, size_of::<StackCallAlloc>());

			(*retv).lock = AllocLock::new();
			if (*retv).lock == null_mut() {
				Self::free_mem(retv);
				return null_mut();
			}
			(*retv).loglvl = ERROR_LEVEL;
			(*retv).fd = ALLOC_DEFAULT_FD;
			retstr = libc::getenv("RSMALLOC_LOGLEVEL\0".as_bytes().as_ptr() as *const libc::c_char);
			if retstr != null_mut() {
				(*retv).loglvl = libc::atoi(retstr);
			}
			retstr = libc::getenv("RSMALLOC_LOGFILE\0".as_bytes().as_ptr() as *const libc::c_char);
			(*retv)._error_write_str("get RSMALLOC_LOGFILE=");
			
			if retstr != null_mut() {
				(*retv)._error_write_val(retstr as u64, true);
				let mut cidx :usize = 0;
				loop {
					let c :libc::c_char = *(retstr.wrapping_add(cidx));
					if c == 0 {
						(*retv)._error_write_str("[");
						(*retv)._error_write_val(c as u64, true);
						(*retv)._error_write_str("]");						
						break;
					}
					(*retv)._error_write_str("[");
					(*retv)._error_write_val(c as u64, true);
					(*retv)._error_write_str("]");
					cidx += 1;
				}
				(*retv)._error_write_str("\n");
				let mut _fd = libc::open(retstr,libc::O_CREAT| libc::O_TRUNC| libc::O_WRONLY,0x1b6);
				(*retv)._error_write_str("opened [");
				(*retv)._error_write_val(_fd as u64, false);
				(*retv)._error_write_str(":");
				(*retv)._error_write_val(_fd as u64, true);
				(*retv)._error_write_str("]\n");
				
				if _fd >= 0 {
					(*retv).fd = _fd;
					_fd = -1;
				}
			} else {
				(*retv)._error_write_str("null\n");
			}

			(*retv).memsize = stacksize;
			(*retv).memlist = libc::malloc(size_of::<*mut MemoryList>() * stacksize) as *mut *mut MemoryList;
			if (*retv).memlist == null_mut() {
				Self::free_mem(retv);
				return null_mut();
			}
			libc::memset((*retv).memlist as *mut libc::c_void, 0, size_of::<*mut MemoryList>() * stacksize);
			retv			
		}
	}

	pub unsafe fn _get_mem_info2(&self) -> Result<MemoryInfo,Box<dyn Error>> {
		let ores = _get_mem_info();
		if ores.is_ok() {
			let info :MemoryInfo = ores.unwrap();
			let mut idx :usize = 0;
			(*self.lock).lock();
			for v in info.maps.iter() {
				self._error_file_line(file!(),line!());
				self._error_write_str("memorymap[");
				self._error_write_val(idx as u64, false);
				self._error_write_str("] [");
				self._error_write_val(v.startaddr as u64, true);
				self._error_write_str("] - [");
				self._error_write_val(v.endaddr as u64, true);
				self._error_write_str("] [");
				self._error_write_str(&v.mapfile);
				self._error_write_str("]\n");
				idx += 1;
			}
			(*self.lock).unlock();

			return Ok(info);
		}
		return ores;
	}


	pub unsafe fn scan(&self) -> i32 {
		let mut idx :usize;
		let mut jdx :usize;
		let mut kdx :usize;
		let mut errcnt :i32 = 0;
		(*self.lock).lock();
		idx = 0;
		while idx < self.memsize {
			let mut cptr :*mut MemoryList = *self.memlist.wrapping_add(idx);
			while cptr != null_mut() {
				self._error_file_line(file!(),line!());
				self._error_write_str("memlist[");
				self._error_write_val(idx as u64,false);
				self._error_write_str("] alignptr[");
				self._error_write_val((*cptr).alignptr as u64,true);
				self._error_write_str("] realptr[");
				self._error_write_val((*cptr).realptr as u64,true);
				self._error_write_str("] size [");
				self._error_write_val((*cptr).size as u64, true);
				self._error_write_str("] align [");
				self._error_write_val((*cptr).alignsize as u64, true);
				self._error_write_str("] backs size [");
				self._error_write_val((*cptr).callsize as u64, false);
				jdx = 0;
				self._error_write_str("]callstack[");
				while jdx < (*cptr).callsize {
					let curback :*const libc::c_void = *((*cptr).callstack.wrapping_add(jdx));
					if jdx > 0 {
						self._error_write_str(",");
					}
					self._error_write_val(curback as u64,true);
					jdx += 1;
				}
				self._error_write_str("]\n");
				/*now to get the stack pointer*/
				jdx = 0;				
				while jdx < (*cptr).callsize {
					let curback :*const libc::c_void = *((*cptr).callstack.wrapping_add(jdx));
					self._error_file_line(file!(),line!());	
					self._error_write_str("pointer[");
					self._error_write_val(curback as u64,true);
					self._error_write_str("] ");
					kdx = 0;
					let mut rptr :*const libc::c_uchar = curback as *const libc::c_uchar;
					while kdx < 16 && ((rptr as u64) % 0x1000) != 0 {
						if kdx > 0 {
							self._error_write_str(" ");
						}
						self._error_write_val(*rptr as u64, true);
						rptr = rptr.wrapping_add(1);
						kdx += 1;
					}
					self._error_write_str("\n");

					jdx += 1;
				}
				errcnt += 1;
				cptr = (*cptr).next;
			}
			idx += 1;
		}

		(*self.lock).unlock();
		return errcnt;
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
    	retv = self._alloc_inner(ptr,retptr,&layout);
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
    	if retv == 0 {
    		(*self)._error_file_line(file!(),line!());
    		(*self)._error_write_str("missing ptr[");
    		(*self)._error_write_val(_ptr as u64, true);
    		(*self)._error_write_str("]\n");    		
    	}
    	(*self.lock).unlock();
    	if retv == 0 {
    		libc::free(_ptr as *mut libc::c_void);
    	}
    	return;

    }
}


fn get_allocator(memsize :usize) -> *mut StackCallAlloc {
	unsafe {
		if GLBL_ALLOC == null_mut() {
			GLBL_ALLOC= StackCallAlloc::new(memsize);
		}
		GLBL_ALLOC		
	}

}

impl StackCallAllocEx {
	pub fn scan(&self) -> i32 {
		let ptr :*mut StackCallAlloc = get_allocator(self.memsize);
		if ptr == null_mut() {
			return 0;
		}
		unsafe {
			return (*ptr).scan();	
		}		
	}

	pub fn get_memory_info(&self) -> Result<MemoryInfo,Box<dyn Error>> {
		let ptr :*mut StackCallAlloc = get_allocator(self.memsize);
		if ptr == null_mut() {
			rsmalloc_new_error!{RsAllocError,"can not get StackCallAlloc"}
		}
		unsafe {
			return (*ptr)._get_mem_info2();
		}
	}
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for StackCallAllocEx {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    	let ptr :*mut StackCallAlloc = get_allocator(self.memsize);
    	if ptr == null_mut() {
    		return null_mut();
    	}
    	return (*ptr).alloc(layout);
    }


    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    	let ptr :*mut StackCallAlloc = get_allocator(self.memsize);
    	if ptr == null_mut() {
    		return;
    	}
    	return (*ptr).dealloc(_ptr,_layout);
    	
    }
}