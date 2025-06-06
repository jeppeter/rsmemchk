
use regex;
use std::io::Read;

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
	let mut n :usize = bksize;
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

	libc::memset(pv as *mut libc::c_void, 0, size_of::<*mut libc::c_void>() * bksize);
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

fn _parse_u64(instr :&str) -> Result<u64,Box<dyn Error>> {
	let mut cparse = format!("{}",instr);
	let mut base :u32 = 10;
	let retv :u64;
	if cparse.starts_with("0x") || cparse.starts_with("0X") {
		cparse = cparse[2..].to_string();
		base = 16;
	} else if cparse.starts_with("x") || cparse.starts_with("X") {
		cparse = cparse[1..].to_string();
		base = 16;
	}

	match u64::from_str_radix(&cparse,base) {
		Ok(v) => {
			retv = v;
		},
		Err(e) => {
			rsmemchk_new_error!{RsAllocError, "parse [{}] error [{:?}]", instr, e}
		}
	}
	Ok(retv)
}


fn _read_file(fname :&str) -> Result<String,Box<dyn Error>> {
	if fname.len() == 0 {
		let f = std::io::stdin();
		let mut reader = std::io::BufReader::new(f);
		let mut retv :String = String::new();
		let res = reader.read_to_string(&mut retv);
		if res.is_err() {
			let err = res.err().unwrap();
			rsmemchk_new_error!{RsAllocError,"read [{}] error [{:?}]", fname,err}
		}
		Ok(retv)
	} else {
		let fo = std::fs::File::open(fname);
		if fo.is_err() {
			let err = fo.err().unwrap();
			rsmemchk_new_error!{RsAllocError,"can not open [{}] error[{:?}]", fname, err}
		}
		let f = fo.unwrap();
		let mut reader = std::io::BufReader::new(f);
		let mut retv :String = String::new();
		let res = reader.read_to_string(&mut retv);
		if res.is_err() {
			let err = res.err().unwrap();
			rsmemchk_new_error!{RsAllocError,"read [{}] error [{:?}]", fname,err}
		}

		Ok(retv)		
	}
}


unsafe fn _get_mem_info() -> Result<MemoryInfo,Box<dyn Error>> {
	let c :String = _read_file("/proc/self/maps")?;
	let sarr : Vec<&str> = c.split("\n").collect();
	let regstr :String = "^([0-9a-fA-F]+)\\-([0-9a-fA-F]+)\\s+([^ ]+)\\s+([0-9a-fA-F]+)\\s+([0-9a-fA-F:]+)\\s+([0-9]+)(\\s+(.*))?".to_string();
	let reg : regex::Regex;
	let ores = regex::Regex::new(&regstr);
	let mut retinfo :MemoryInfo = MemoryInfo::new();
	let mut curmap :MemoryMap;
	let mut curs :String;

	if ores.is_err() {
		rsmemchk_new_error!{RsAllocError,"[{}] compile error {:?}", regstr, ores.err().unwrap()}
	}
	reg = ores.unwrap();
	for s in sarr.iter() {
		let ocap = reg.captures(s);
		if ocap.is_some() {
			let cap = ocap.unwrap();
			if cap.len() >= 9  {
				curmap = MemoryMap::new();
				curs = format!("0x{}",cap.get(1).map_or("", |m| m.as_str()));
				curmap.startaddr = _parse_u64(&curs)?;
				curs = format!("0x{}",cap.get(2).map_or("", |m| m.as_str()));
				curmap.endaddr = _parse_u64(&curs)? - 1;

				curs = format!("{}",cap.get(3).map_or("", |m| m.as_str()));
				let curb :&[u8] = curs.as_bytes();
				let mut idx:usize = 0;
				while idx < curb.len() {
					if curb[idx] == b'r' {
						curmap.protect |= MEM_READ;
					} else if curb[idx] == b'w' {
						curmap.protect |= MEM_WRITE;
					} else if curb[idx] == b'x' {
						curmap.protect |= MEM_EXEC;
					}
					idx += 1;
				}

				if cap[8].len() > 0 {
					let curs = format!("{}",cap.get(8).map_or("", |m| m.as_str()));
					if curs.as_bytes()[0] == '/' as u8 {
						curmap.mapfile = format!("{}",curs);	
					}					
				} 
				retinfo.maps.push(curmap);
			}
		}
	}
	Ok(retinfo)
}