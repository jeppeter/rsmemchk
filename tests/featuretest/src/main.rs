
#[cfg(feature="rsmalloc_mode")]
use rsmalloc::{StackCallAllocEx,MemoryInfo};
use std::mem::ManuallyDrop;
use std::error::Error;
use extargsparse_worker::{extargs_new_error,extargs_error_class};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::cell::UnsafeCell;
use caller::{call_function,cc_func};
use rsmalloc::{cfg_rsmalloc_not_inline};


extargs_error_class!{ArcError}


#[cfg(feature="rsmalloc_mode")]
#[global_allocator]
static ALLOCATOR: StackCallAllocEx = StackCallAllocEx{memsize : 23, stacksize : 8};

#[derive(Debug)]
struct CB {
	cc :i32,
}

#[derive(Debug)]
struct DB {
	bb :Vec<i32>,
}


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
			//let mut ov :Option<Rc<RefCell<CFunc>>> = None;
			let mut ks :String = "".to_string();
			for (k,_v) in self.callfuncs.borrow().iter() {
				println!("k {}", k);
				ks = format!("{}",k);
				idx += 1;
				break;
			}

			if idx == 0 {
				break;
			}
			let ov = self.callfuncs.borrow_mut().remove(&ks);
			let c = ov.unwrap();
			println!("rc count {}",Rc::strong_count(&c));
			drop(c);
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
	cfg_rsmalloc_not_inline !{
		fn new(val :i32) -> Result<Self,Box<dyn Error>> {
			Ok(Self {
				inner : Rc::new(RefCell::new(CInner::new(val)?)),
			})
		}

		fn call_load(&self, v :&str,vn :&str) -> Result<(),Box<dyn Error>> {
			return self.inner.borrow_mut().call_load(v,vn);
		}		
	}
}

impl CInner {
	cfg_rsmalloc_not_inline !{
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
}


cfg_rsmalloc_not_inline! {





	fn call_1(c :CB) -> DB {
		let mut d = DB {
			bb :vec![],
		};

		while d.bb.len() < c.cc as usize {
			d.bb.push(33);
		}
		d
	}

	fn call_2(v :i32) -> CB {
		CB {
			cc :v
		}
	}

	fn call_3(x :i32) -> DB {
		let c :CB = call_2(x);
		return call_1(c);
	}

	fn call_4(c :i32) -> Vec<DB> {
		let mut retv :Vec<DB> = vec![];
		for i in 0..c{
			retv.push(call_3(i));
		}
		retv
	}

}

fn main() -> Result<(),Box<dyn Error>> {
	let c :CB = call_2(77);
	let d :DB = call_3(50);
	let ee :DB = call_3(9);
	let f = call_4(92);
	let e :ManuallyDrop<DB> = ManuallyDrop::new(ee);

	println!("c {:?}\nd {:?}\ne {:?}\n{:?}",c,d,&e,f);
	drop(c);
	drop(d);
	drop(f);

	let c :C = C::new(32)?;
	let b = c.clone();
	c.call_load("hello","new")?;
	drop(c);
	b.call_load("hello","world")?;
	drop(b);

	call_function("cc");
	cc_func("used");

	#[cfg(feature="rsmalloc_mode")]
	ALLOCATOR.scan();
	#[cfg(feature="rsmalloc_mode")]
	let _maps :MemoryInfo = ALLOCATOR.get_memory_info()?;
	Ok(())
}
