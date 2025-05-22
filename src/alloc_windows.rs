fn _write_func(fd :libc::c_int, buf :*const libc::c_void, size :u32)
{
	unsafe {
		let _ = libc::write(fd,buf,size);
		return;
	}
}


use winapi::um::minwinbase::{CRITICAL_SECTION};
use winapi::um::synchapi::{InitializeCriticalSection,EnterCriticalSection,LeaveCriticalSection};
use winapi::um::winnt::{RtlCaptureStackBackTrace};
use winapi::shared::minwindef::{ULONG,WORD};

const SKIP_WIN_BKSIZE :usize = 1;

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _get_stack_call(skip :usize,pv :*mut * mut libc::c_void,bksize :usize) -> i32 {
	let mut realbacks :*mut *mut libc::c_void = null_mut();
	let mut n :usize = 4;
	let mut retv :i32 = 0;
	let mut sret :WORD;
	let mut nret :ULONG;

	loop {
		if realbacks != null_mut() {
			libc::free(realbacks as *mut libc::c_void);
		}
		realbacks = libc::malloc(size_of::<*mut libc::c_void>() * n) as *mut *mut libc::c_void;
		if realbacks == null_mut() {
			/*for error*/
			return -1;
		}
		nret = 0;
		let _ptr :*mut ULONG = &mut nret;
		sret = RtlCaptureStackBackTrace(0,n as u32,realbacks as *mut *mut winapi::ctypes::c_void,_ptr);
		if (sret as usize) < n {
			break;
		}
		n <<= 1;
	}

	for i in SKIP_WIN_BKSIZE..n {
		if (i-SKIP_WIN_BKSIZE) >= skip && (i-SKIP_WIN_BKSIZE-skip) < bksize {
			retv += 1;
			(*pv.wrapping_add(i-SKIP_WIN_BKSIZE-skip)) = *realbacks.wrapping_add(i);
		}
	}
	libc::free(realbacks as *mut libc::c_void);

	return retv;
}


#[repr(C)]
struct AllocLock {	
	cs :*mut CRITICAL_SECTION,
}

#[allow(unsafe_op_in_unsafe_fn)]
#[allow(dead_code)]
impl AllocLock {
	unsafe fn new() -> *mut AllocLock {
		let retv :*mut AllocLock = libc::malloc(size_of::<AllocLock>()) as *mut AllocLock;
		if retv == null_mut() {
			return null_mut();
		}
		libc::memset(retv as *mut libc::c_void, 0,size_of::<AllocLock>());
		(*retv).cs = libc::malloc(size_of::<CRITICAL_SECTION>()) as *mut CRITICAL_SECTION;
		if (*retv).cs == null_mut() {
			Self::free_mem(retv);
			return null_mut();
		}
		InitializeCriticalSection((*retv).cs);
		retv
	}

	unsafe fn lock(&mut self) {
		EnterCriticalSection(self.cs as *mut CRITICAL_SECTION);
	}

	unsafe fn unlock(&mut self) {
		LeaveCriticalSection(self.cs as *mut CRITICAL_SECTION);
	}

	fn free_mem(retv :*mut AllocLock) {
		unsafe {
			if retv != null_mut() {
				if (*retv).cs != null_mut() {
					libc::free((*retv).cs as *mut libc::c_void);
				}
				(*retv).cs = null_mut();

				libc::free(retv as *mut libc::c_void);
			}
		}
		return;
	}
}