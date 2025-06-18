# rsmemgen
> rust memory leak check code generation library

### Release History
* Jun 18th 2025 to release 0.1.0 for first version

> Cargo.toml
```cargo
[package]
name = "cases"
version = "0.1.0"
edition = "2021"

[dependencies]
rsmemgen = "0.1.0"

[features]
rsmemchk_mode = []
debug_mode = []
```
>build.rs
```rust

use std::error::Error;



fn main() -> Result<(),Box<dyn Error>> {
	match std::env::var("CARGO_CFG_FEATURE") {
		Ok(v) => {
			let carrs :Vec<&str> = v.split(",").collect();
			let mut idx :usize = 0;
			while idx < carrs.len() {
				if carrs[idx] == "rsmemchk_mode" {
					/*to set for RSMEMCHK_MODE for it will set environment build*/
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
```

> main.rs
```rust
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
```
> call in cargo
```shell
cargo build --release
```

> result
> call_hello or Values::call_name Values::call_user will compiled into inline

> call in cargo
```shell
cargo build --release --features rsmemchk_mode
```

> result
> call_hello or Values::call_name Values::call_user will compiled not inlined it will help you when used rsmemchk functions


