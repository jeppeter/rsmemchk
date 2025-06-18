
use std::error::Error;



fn main() -> Result<(),Box<dyn Error>> {
	match std::env::var("CARGO_CFG_FEATURE") {
		Ok(v) => {
			let carrs :Vec<&str> = v.split(",").collect();
			let mut idx :usize = 0;
			while idx < carrs.len() {
				if carrs[idx] == "rsmemchk_mode" {
					println!("cargo::rustc-env=RSMEMCHK_MODE=1");					
					break;
				}
				idx += 1;
			}
		},
		Err(_e) => {},
	}

	
	Ok(())
}