fn _write_func(fd :libc::c_int, buf :*const libc::c_void, size :u32)
{
	unsafe {
		let _ = libc::write(fd,buf,size);
		return;
	}
}


use winapi::um::minwinbase::{CRITICAL_SECTION};
//use winapi::shared::basetsd::{ULONG_PTR};
use winapi::um::synchapi::{InitializeCriticalSection,EnterCriticalSection,LeaveCriticalSection};
#[allow(unused_imports)]
use winapi::um::winnt::{RtlCaptureStackBackTrace,HANDLE,PVOID,PROCESS_VM_READ,PROCESS_QUERY_INFORMATION};
#[allow(unused_imports)]
use winapi::shared::minwindef::{ULONG,WORD,BOOL,DWORD,TRUE,LPVOID,FALSE};
#[allow(unused_imports)]
use winapi::um::processthreadsapi::{OpenProcess,GetCurrentProcessId,GetCurrentProcess};
use winapi::um::psapi::{QueryWorkingSet,PSAPI_WORKING_SET_INFORMATION,PSAPI_WORKING_SET_BLOCK,GetMappedFileNameA};
use winapi::um::handleapi::{CloseHandle};

const SKIP_WIN_BKSIZE :usize = 1;

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn _get_stack_call(skip :usize,pv :*mut * mut libc::c_void,bksize :usize) -> i32 {
	let sret :WORD;
	let mut nret :ULONG;

	libc::memset(pv as *mut libc::c_void,0, size_of::<*mut libc::c_void>() * bksize);
	nret = 0;
	let _ptr :*mut ULONG = &mut nret;
	sret = RtlCaptureStackBackTrace((SKIP_WIN_BKSIZE+skip) as u32,bksize as u32,pv as  *mut *mut winapi::ctypes::c_void,_ptr);
	return sret as i32;

	//let mut retv :i32 = 0;
	//let mut realbacks :*mut *mut libc::c_void = null_mut();
	//let mut n :usize = bksize;
	// loop {
	// 	if realbacks != null_mut() {
	// 		libc::free(realbacks as *mut libc::c_void);
	// 	}
	// 	realbacks = libc::malloc(size_of::<*mut libc::c_void>() * n) as *mut *mut libc::c_void;
	// 	if realbacks == null_mut() {
	// 		/*for error*/
	// 		return -1;
	// 	}
	// 	nret = 0;
	// 	let _ptr :*mut ULONG = &mut nret;
	// 	sret = RtlCaptureStackBackTrace(0,n as u32,realbacks as *mut *mut winapi::ctypes::c_void,_ptr);
	// 	if (sret as usize) < n {
	// 		break;
	// 	}
	// 	n <<= 1;
	// }

	// for i in SKIP_WIN_BKSIZE..n {
	// 	if (i-SKIP_WIN_BKSIZE) >= skip && (i-SKIP_WIN_BKSIZE-skip) < bksize {
	// 		retv += 1;
	// 		(*pv.wrapping_add(i-SKIP_WIN_BKSIZE-skip)) = *realbacks.wrapping_add(i);
	// 	}
	// }
	// libc::free(realbacks as *mut libc::c_void);

	// return retv;
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

const WIN_PAGE_SHIFT :u64 = 12;
const WIN_PAGE_ADDR_MASK :u64 = (1 << WIN_PAGE_SHIFT) - 1;
//const WIN_PAGE_ADDR_ALIGN :u64 = !(WIN_PAGE_ADDR_MASK);
const FNAME_SIZE :usize = 256;

unsafe fn get_len(bstr :&[u8;FNAME_SIZE]) -> u32 {
	let mut idx :usize = 0;
	while idx < FNAME_SIZE {
		if bstr[idx] == 0 {
			return idx as u32;
		}
		idx += 1;
	}
	return idx as u32;
}

fn _get_prot(val :u32) -> u32 {
	match val {
		0|8 |16 | 24 => {
			return 0;
		},
		1|9 | 17 | 25 => {
			return MEM_READ;
		},
		2|10 | 18 | 26 => {
			return MEM_EXEC;
		},
		3|11 | 19 | 27=> {
			return MEM_WRITE|MEM_READ;
		},
		4|12 | 20 | 28=> {
			return MEM_WRITE|MEM_READ;
		},
		5|13 | 21 | 29=> {
			return MEM_WRITE|MEM_READ;	
		},
		6|14 | 22 | 30=> {
			return MEM_WRITE|MEM_READ|MEM_EXEC;
		},
		7|15 | 23 | 31=> {
			return MEM_WRITE|MEM_READ|MEM_EXEC;	
		},
		_ => {
			return 0;
		}
	}
}



#[allow(unused_assignments)]
#[allow(unused_mut)]
unsafe fn _get_mem_info() -> Result<MemoryInfo,Box<dyn Error>> {
	let mut hproc :HANDLE = null_mut() ;
	let mut cinfo :*mut PSAPI_WORKING_SET_INFORMATION = null_mut();
	let mut cinfosize :usize = size_of::<PSAPI_WORKING_SET_INFORMATION>();
	let mut lastpage :u64 = 0;
	let mut retinfo :MemoryInfo = MemoryInfo::new();
	let mut curmap :MemoryMap = MemoryMap::new();
	let mut saddr :u64;
	let mut bret :BOOL;
	let mut filename :[i8;FNAME_SIZE] = [0;FNAME_SIZE];
	let mut storefilename :[u8;FNAME_SIZE] = [0;FNAME_SIZE];
	let mut sret :DWORD;
	let mut cptr :*mut i8 = null_mut();
	let mut sptr :*const i8;
	let mut lastprotect :u32 = 0;

	//pid = GetCurrentProcessId();

	//hproc = OpenProcess(PROCESS_VM_READ|PROCESS_QUERY_INFORMATION,FALSE,pid);
	//if hproc == null_mut() {
	//	rsmalloc_new_error!{RsAllocError, "can not open {} error", pid}
	//}
	hproc = GetCurrentProcess();

	loop {
		if cinfo != null_mut() {
			libc::free( cinfo as *mut libc::c_void);
		}
		cinfo = libc::malloc(cinfosize) as *mut PSAPI_WORKING_SET_INFORMATION;
		if cinfo == null_mut() {
			if hproc != GetCurrentProcess() {
				CloseHandle(hproc);	
			}			
			rsmalloc_new_error!{RsAllocError,"can not alloc size {}", cinfosize}
		}

		bret = QueryWorkingSet(hproc,cinfo as PVOID,cinfosize as u32);
		if bret == TRUE {
			break;
		}

		cinfosize = size_of::<PSAPI_WORKING_SET_INFORMATION>() + (*cinfo).NumberOfEntries * size_of::<PSAPI_WORKING_SET_BLOCK>();
	}

	/*now we should give the memory*/
	let wkset :*const PSAPI_WORKING_SET_BLOCK = &((*cinfo).WorkingSetInfo[0]) as *const PSAPI_WORKING_SET_BLOCK;
	rsmalloc_debug_buffer_trace!(cinfo, size_of::<PSAPI_WORKING_SET_INFORMATION>()+((*cinfo).NumberOfEntries - 1) * size_of::<PSAPI_WORKING_SET_BLOCK>(),"TOTAL workingset");
	for i in 0..(*cinfo).NumberOfEntries {
		let cblock :*const PSAPI_WORKING_SET_BLOCK = wkset.wrapping_add(i) as *const PSAPI_WORKING_SET_BLOCK;
		let curpage :u64 = ((*cblock).Flags >> 12) as u64;
		let curprot :u32 = _get_prot(((*cblock).Flags & 0x1f) as u32);
		rsmalloc_debug_buffer_trace!(cblock, size_of::<PSAPI_WORKING_SET_BLOCK>(), "{} cblock VirtualPage 0x{:x}",i,curpage);
		if i == 0 {
			saddr = (curpage as u64) << WIN_PAGE_SHIFT;
			lastpage = curpage as u64;
			curmap = MemoryMap::new();
			curmap.startaddr = saddr;
			curmap.protect = curprot;
			cptr = (&mut filename) as *mut i8;
			libc::memset(cptr as *mut libc::c_void,0, FNAME_SIZE);
			cptr = (&mut filename) as *mut i8;
			sret =  GetMappedFileNameA(hproc,saddr as LPVOID,cptr,FNAME_SIZE as u32);
			rsmalloc_log_trace!("[{}]saddr 0x{:x} sret {}",i, saddr, sret);
			if sret == 0 {
				cptr = ((&mut storefilename) as *mut u8) as *mut i8;
				libc::memset(cptr as *mut libc::c_void,0,FNAME_SIZE);
			} else {
				cptr = ((&mut storefilename) as *mut u8) as *mut i8;
				sptr = &filename as *const i8;
				libc::memcpy(cptr as *mut libc::c_void,sptr as *const libc::c_void,FNAME_SIZE);
				sret = get_len(&storefilename);
				curmap.mapfile = String::from_utf8_lossy(&storefilename[0..(sret as usize )]).to_string();
			}
		} else {
			saddr = (curpage as u64) << WIN_PAGE_SHIFT;
			if (lastpage+1) == curpage as u64 && lastprotect == curprot {
				cptr = &mut filename as *mut i8;
				sret = GetMappedFileNameA(hproc,saddr as LPVOID,cptr,FNAME_SIZE as u32);
				rsmalloc_log_trace!("[{}]saddr 0x{:x} sret {}",i, saddr, sret);
				if sret == 0 {
					if storefilename[0] != 0 {
						curmap.endaddr = (lastpage << WIN_PAGE_SHIFT) + WIN_PAGE_ADDR_MASK;
						retinfo.maps.push(curmap);
						curmap = MemoryMap::new();
						curmap.startaddr = saddr;
						curmap.protect = curprot;
						cptr = ((&mut storefilename) as *mut u8) as *mut i8;
						libc::memset(cptr as *mut libc::c_void,0,FNAME_SIZE);						
					}
				} else {
					sptr = (&storefilename as *const u8) as *const i8;
					if libc::strcmp(&filename as *const i8,sptr) != 0 {
						curmap.endaddr = (lastpage << WIN_PAGE_SHIFT) + WIN_PAGE_ADDR_MASK;
						retinfo.maps.push(curmap);
						curmap = MemoryMap::new();
						curmap.startaddr = saddr;
						curmap.protect = curprot;
						cptr = (&mut storefilename as *mut u8 ) as *mut i8;
						sptr = &filename as *const i8;
						libc::memcpy(cptr as *mut libc::c_void,sptr as *const libc::c_void,FNAME_SIZE);
						sret = get_len(&storefilename);
						curmap.mapfile = String::from_utf8_lossy(&storefilename[0..(sret as usize)]).to_string();
					}
				}				
			} else {
				curmap.endaddr = (lastpage << WIN_PAGE_SHIFT) + WIN_PAGE_ADDR_MASK;
				retinfo.maps.push(curmap);
				curmap = MemoryMap::new();
				curmap.startaddr = saddr;
				curmap.protect = curprot;
				sret = GetMappedFileNameA(hproc,saddr as LPVOID,cptr,FNAME_SIZE as u32);
				rsmalloc_log_trace!("[{}]saddr 0x{:x} sret {}",i, saddr, sret);
				if sret == 0 {
					cptr = ((&mut storefilename) as *mut u8) as *mut i8;
					libc::memset(cptr as *mut libc::c_void,0,FNAME_SIZE);
				} else {
					cptr = (&mut storefilename as *mut u8 ) as *mut i8;
					sptr = &filename as *const i8;
					libc::memcpy(cptr as *mut libc::c_void,sptr as *const libc::c_void,FNAME_SIZE);
					sret = get_len(&storefilename);
					rsmalloc_debug_buffer_trace!(cptr, sret, "copy name");
					curmap.mapfile = String::from_utf8_lossy(&storefilename[0..(sret as usize)]).to_string();
				}
			}
			lastpage = curpage as u64;
		}
		lastprotect = curprot;
	}

	libc::free(cinfo as *mut libc::c_void);
	if hproc != GetCurrentProcess() {
		CloseHandle(hproc);	
	}	
	Ok(retinfo)
}