
use rsmemgen::{rsmemgen_func_inline};

#[rsmemgen_func_inline()]
pub fn call_function(n :&str) {
	println!("caller {}", n);
}	


#[rsmemgen_func_inline()]
pub fn cc_func(n :&str) {
	println!("cc_func  {}", n);
}

