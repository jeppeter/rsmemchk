use std::error::Error;
use rsmemgen::{rsmemchk_inline_attr};


struct Values {
	pub val :i32,
}

#[rsmemchk_inline_attr()]
impl Values {
	#[inline(never)]
	pub fn new(val :i32) ->  Self {
		Self {
			val :val,
		}
	}

	pub fn to_string(&self) -> String {
		println!("in line [{}:{}]",file!(),line!());
		return format!("{}",self.val);
	}
}

fn main() -> Result<(),Box<dyn Error>> {
    let c = Values::new(32);
    println!("{}", c.to_string());
    Ok(())
}
