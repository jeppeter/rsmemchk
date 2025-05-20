

struct AllocLock {

}

impl AllocLock {
	fn new() -> Self {
		Self{}
	}

	fn lock(&mut self) {

	}

	fn unlock(&mut self) {

	}

	fn free_mem(retv :*const AllocLock) {
		if retv != std::mem::null_ptr() {
			libc::free(retv);
		}
		return;
	}
}