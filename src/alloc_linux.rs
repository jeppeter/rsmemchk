
fn _write_func(fd :libc::c_int, buf :*const libc::c_void, size :u32)
{
	unsafe {
		let _ = libc::write(fd,buf,size as usize);
		return;		
	}
}


const SKIP_UX_BKSIZE :usize = 1;

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _get_stack_call(skip :usize,pv :*mut * mut libc::c_void,bksize :usize) -> i32 {
	let mut realbacks :*mut *mut libc::c_void = null_mut();
	let mut n :usize = 4;
	let mut retv :i32 = 0;
	let mut sret :i32;

	loop {
		if realbacks != null_mut() {
			libc::free(realbacks as *mut libc::c_void);
		}
		realbacks = libc::malloc(size_of::<*mut libc::c_void>() * n) as *mut *mut libc::c_void;
		if realbacks == null_mut() {
			/*for error*/
			return -1;
		}
		sret = libc::backtrace(realbacks,n as i32);
		if sret < 0 {
			libc::free(realbacks as *mut libc::c_void);
			return -1;
		} else if sret < n as i32 {
			break;
		}
		n <<= 1;
	}

	for i in SKIP_UX_BKSIZE..n {
		if (i-SKIP_UX_BKSIZE) >= skip && (i-SKIP_UX_BKSIZE-skip) < bksize {
			retv += 1;
			(*pv.wrapping_add(i-SKIP_UX_BKSIZE-skip)) = *realbacks.wrapping_add(i);
		}
	}

	libc::free(realbacks as *mut libc::c_void);

	return retv;
}


#[repr(C)]
struct AllocLock {
	lockptr :*mut libc::pthread_mutex_t,
}

#[allow(dead_code)]
impl AllocLock {
	pub unsafe fn free_mem(ptr :*mut AllocLock) {
		if ptr != null_mut() {
			if (*ptr).lockptr != null_mut() {
				libc::free((*ptr).lockptr as *mut libc::c_void);
			}
			(*ptr).lockptr = null_mut();
			libc::free(ptr as *mut libc::c_void);
		}
	}

	pub unsafe fn new() -> *mut AllocLock {
		let retv :*mut AllocLock;

		retv = libc::malloc(size_of::<AllocLock>()) as *mut AllocLock;
		if retv == null_mut() {
			return null_mut();
		}
		libc::memset(retv as *mut libc::c_void, 0 ,size_of::<AllocLock>());
		(*retv).lockptr =  libc::malloc(size_of::<libc::pthread_mutex_t>()) as *mut libc::pthread_mutex_t;
		if (*retv).lockptr == null_mut() {
			Self::free_mem(retv);
			return null_mut();
		}
		libc::pthread_mutex_init((*retv).lockptr, null_mut());
		retv
	}

	fn lock(&self) {
		unsafe {
			libc::pthread_mutex_lock(self.lockptr);	
		}		
	}

	fn unlock(&self) {
		unsafe {
			libc::pthread_mutex_unlock(self.lockptr);	
		}		
	}
}