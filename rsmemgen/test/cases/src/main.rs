use std::error::Error;
use rsmemgen::{rsmemgen_impl_inline,rsmemgen_func_inline};


struct Values {
	pub val :i32,
}

trait CallUser {
	fn call_name(&self,s :&str);
	fn call_user(&self,s :&str);
}

#[rsmemgen_impl_inline()]
impl Values {
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

#[rsmemgen_impl_inline()]
impl CallUser for Values {
	fn call_name(&self,s :&str) {
		println!("value {} name {}", self.val,s);
	}
	fn call_user(&self,s :&str) {
		println!("value {} user {}", self.val,s);
	}
}

#[rsmemgen_func_inline()]
fn call_hello(s :&str) {
	println!("hello {}", s);
}

fn main() -> Result<(),Box<dyn Error>> {
    let c = Values::new(32);
    println!("{}", c.to_string());
    c.call_user("user");
    c.call_name("user");
    call_hello("newguest");
    Ok(())
}
