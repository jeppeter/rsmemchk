
use winapi::um::minwinbase::{CRITICAL_SECTION};
use winapi::um::synchapi::{InitializeCriticalSection,EnterCriticalSection,LeaveCriticalSection};

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