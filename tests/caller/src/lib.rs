
use rsmalloc::cfg_rsmalloc_not_inline;

cfg_rsmalloc_not_inline!{
	pub fn call_function(n :&str) {
		println!("caller {}", n);
	}	
}


#[cfg(feature="rsmalloc_mode")]
pub fn cc_func(n :&str) {
	println!("cc_func with rsmalloc_mode {}", n);
}

#[cfg(not(feature="rsmalloc_mode"))]
pub fn cc_func(n :&str) {
	println!("cc_func with not rsmalloc_mode {}", n);
}
