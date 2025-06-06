
use rsmemchk::{StackCallAllocEx,MemoryInfo};
use std::mem::ManuallyDrop;
use std::error::Error;

#[global_allocator]
static ALLOCATOR: StackCallAllocEx = StackCallAllocEx{memsize : 23,stacksize :8};

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


fn main() -> Result<(),Box<dyn Error>> {
	let c :C = call_2(77);
	let d :D = call_3(50);
	let ee :D = call_3(9);
	let e :ManuallyDrop<D> = ManuallyDrop::new(ee);

	println!("c {:?}\nd {:?}\ne {:?}",c,d,&e);
	drop(c);
	ALLOCATOR.scan();
	let maps :MemoryInfo = ALLOCATOR.get_memory_info()?;
	for v in maps.maps.iter() {
		println!("0x{:x} - 0x{:x} [{}]", v.startaddr,v.endaddr,v.mapfile);
	}
	Ok(())
}