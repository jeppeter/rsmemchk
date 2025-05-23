
use rsmalloc::{StackCallAllocEx};
use std::mem::ManuallyDrop;

#[global_allocator]
static ALLOCATOR: StackCallAllocEx = StackCallAllocEx{};

#[derive(Debug)]
struct C {
	cc :i32,
}

#[derive(Debug)]
struct D {
	bb :Vec<i32>,
}

fn call_1(c :C) -> D {
	let mut d = D {
		bb :vec![],
	};

	while d.bb.len() < c.cc as usize {
		d.bb.push(33);
	}
	d
}

fn call_2(v :i32) -> C {
	C {
		cc :v
	}
}

fn call_3(x :i32) -> D {
	let c :C = call_2(x);
	return call_1(c);
}


fn main() {
	let c :C = call_2(77);
	let d :D = call_3(50);
	let e :ManuallyDrop<D> = ManuallyDrop::new(d);

	println!("c {:?}\ne {:?}",c,&e);
	drop(c);
	StackCallAllocEx::scan();
}