use proc_macro::TokenStream;

#[macro_use]
mod errors;
#[macro_use]
mod logger;

use quote::{ToTokens};

use logger::{rsmemgen_debug_out,rsmemgen_log_get_timestamp};


macro_rules! syn_error_fmt {
	($($a:expr),*) => {
		let cerr = format!($($a),*);
		eprintln!("{}",cerr);
		rsmemgen_log_error!("{}",cerr);
		return cerr.parse().unwrap();
		//return syn::Error::new(
        //            Span::call_site(),
        //            $cerr,
        //        ).to_compile_error().to_string().parse().unwrap();
    }
}


#[proc_macro_attribute]
pub fn rsmemchk_inline_attr(_args :TokenStream , input :TokenStream) -> TokenStream {
	let mut implstruct : syn::ItemImpl ;
	match syn::parse::<syn::ItemImpl>(input.clone()) {
		Ok(v) => {
			implstruct = v.clone();
		},
		Err(_e) => {
			syn_error_fmt!("not parse \n{}",input.to_string());
          	//return syn::Error::new(
            //        Span::call_site(),
            //        &format!(
            //            "not parse \n{}",
            //            item.to_string()
            //        ),
            //    ).to_compile_error().to_string().parse().unwrap();

        }
    }

    for item in implstruct.items.iter_mut() {
        let mut tk : TokenStream;
        tk = item.into_token_stream().into();
    	rsmemgen_log_trace!("attr {}",tk.to_string());
    }

    return input;

}