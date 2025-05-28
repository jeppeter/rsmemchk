

use rsmalloc::{StackCallAllocEx,MemoryInfo};
use std::rc::Rc;
use std::cell::{RefCell,UnsafeCell};
#[allow(unused_imports)]
use extargsparse_worker::{extargs_error_class,extargs_new_error};
use std::error::Error;
use std::collections::HashMap;
use std::sync::Arc;

extargs_error_class!{ArcError}

#[global_allocator]
static ALLOCATOR: StackCallAllocEx = StackCallAllocEx{memsize : 23,stacksize :16};

#[derive(Clone)]
enum CFunc {
	LoadFunc(Rc<dyn Fn(&str) -> Result<(),Box<dyn Error>>>),
}



#[derive(Clone)]
struct CInner {
	val :i32,
	callfuncs :Rc<RefCell<HashMap<String,Rc<RefCell<CFunc>>>>>,
}

impl Drop for CInner {
	fn drop(&mut self) {
		println!("CInner drop");
		loop {
			let mut idx :usize = 0;
			let mut ov :Option<Rc<RefCell<CFunc>>> = None;
			for (k,v) in self.callfuncs.borrow_mut().iter() {
				println!("k {}", k);
				ov = self.callfuncs.borrow_mut().remove(k);
				idx += 1;
				break;
			}

			if idx == 0 {
				break;
			}
			let c =ov.unwrap();
			println!("rc count {}",Rc::strong_count(&c));
			drop(c);
		}
	}
}

impl CInner {
	fn _add_funcs(&mut self) -> Result<(),Box<dyn Error>> {
		let b = Arc::new(UnsafeCell::new(self.clone()));
		let mut bmut =  self.callfuncs.borrow_mut();
		let s1 = b.clone();
		bmut.insert(format!("hello",),Rc::new(RefCell::new(CFunc::LoadFunc(Rc::new(move |n| {let  c :&mut CInner = unsafe {&mut *s1.get()};
			c.hello_func(n)
		} )))));
		Ok(())
	}

	fn hello_func(&mut self, n :&str ) -> Result<(),Box<dyn Error>> {
		println!("val[{}]hello {}",self.val, n);
		Ok(())
	}

	fn new(val :i32) -> Result<Self,Box<dyn Error>> {
		let mut retv :Self = Self {
			val :val,
			callfuncs :Rc::new(RefCell::new(HashMap::new())),
		};
		let _ = retv._add_funcs()?;
		Ok(retv)
	}

	fn _get_load_func(&self, v :&str) -> Option<CFunc> {
		let mut retv : Option<CFunc> = None;
		match self.callfuncs.borrow().get(v) {
			Some(f1) => {
				let f2 :&CFunc = &f1.borrow();
				retv = Some(f2.clone());
			},
			None => {}
		}
		retv		
	}

	fn call_load(&mut self, v :&str,vn :&str) -> Result<(),Box<dyn Error>> {
		let fnptr :Option<CFunc>;
		fnptr = self._get_load_func(v);
		if fnptr.is_some() {
			let f2 = fnptr.unwrap();
			match f2 {
				CFunc::LoadFunc(f) => {
					return f(vn);
				},
			}
		} else {
			extargs_new_error!{ArcError,"can not found [{}] load command map function", v}
		}
	}
}

#[derive(Clone)]
struct  C {
	inner :Rc<RefCell<CInner>>,
}

impl Drop for C {
	fn drop(&mut self) {
		println!("C cnt {}",  Rc::strong_count(&self.inner));		
	}
}

impl C {
	fn new(val :i32) -> Result<Self,Box<dyn Error>> {
		Ok(Self {
			inner : Rc::new(RefCell::new(CInner::new(val)?)),
		})
	}

	fn call_load(&self, v :&str,vn :&str) -> Result<(),Box<dyn Error>> {
		return self.inner.borrow_mut().call_load(v,vn);
	}
}

fn main() -> Result<(),Box<dyn Error>> {
	let c :C = C::new(32)?;
	let b = c.clone();
	c.call_load("hello","new")?;
	drop(c);
	b.call_load("hello","world")?;
	drop(b);
	ALLOCATOR.scan();
	let _maps :MemoryInfo = ALLOCATOR.get_memory_info()?;
	Ok(())
}