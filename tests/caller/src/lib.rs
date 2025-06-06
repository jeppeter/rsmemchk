
use rsmemchk::cfg_rsmemchk_not_inline;

cfg_rsmemchk_not_inline!{
	pub fn call_function(n :&str) {
		println!("caller {}", n);
	}	
}


#[cfg(feature="rsmemchk_mode")]
pub fn cc_func(n :&str) {
	println!("cc_func with rsmemchk_mode {}", n);
}

#[cfg(not(feature="rsmemchk_mode"))]
pub fn cc_func(n :&str) {
	println!("cc_func with not rsmemchk_mode {}", n);
}
