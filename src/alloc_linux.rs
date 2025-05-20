

#[repr(C)]
struct AllocLock {	
}

#[allow(dead_code)]
impl AllocLock {
	fn new() -> *mut AllocLock {
		let retv :*mut AllocLock = null_mut();
		retv
	}

	fn lock(&mut self) {

	}

	fn unlock(&mut self) {

	}

	fn free_mem(retv :*mut AllocLock) {
		if retv != null_mut() {
			libc::free(retv);
		}
		return;
	}
}