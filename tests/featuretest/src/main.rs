
#[cfg(feature="rsmalloc_mode")]
use rsmalloc::{StackCallAllocEx,MemoryInfo};
use std::mem::ManuallyDrop;
use std::error::Error;

#[cfg(feature="rsmalloc_mode")]
#[global_allocator]
static ALLOCATOR: StackCallAllocEx = StackCallAllocEx{memsize : 23};

#[derive(Debug)]
struct C {
	cc :i32,
}

#[derive(Debug)]
struct D {
	bb :Vec<i32>,
}

#[inline(never)]
fn call_1(c :C) -> D {
	let mut d = D {
		bb :vec![],
	};

	while d.bb.len() < c.cc as usize {
		d.bb.push(33);
	}
	d
}

#[inline(never)]
fn call_2(v :i32) -> C {
	C {
		cc :v
	}
}

#[inline(never)]
fn call_3(x :i32) -> D {
	let c :C = call_2(x);
	return call_1(c);
}

#[inline(never)]
fn call_4(c :i32) -> Vec<D> {
	let mut retv :Vec<D> = vec![];
	for i in 0..c{
		retv.push(call_3(i));
	}
	retv
}


fn main() -> Result<(),Box<dyn Error>> {
	let c :C = call_2(77);
	let d :D = call_3(50);
	let ee :D = call_3(9);
	let f = call_4(92);
	let e :ManuallyDrop<D> = ManuallyDrop::new(ee);

	println!("c {:?}\nd {:?}\ne {:?}\n{:?}",c,d,&e,f);
	drop(c);
	drop(d);
	drop(f);
	#[cfg(feature="rsmalloc_mode")]
	ALLOCATOR.scan();
	#[cfg(feature="rsmalloc_mode")]
	let _maps :MemoryInfo = ALLOCATOR.get_memory_info()?;
	Ok(())
}