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


#[allow(unused_assignments)]
#[proc_macro_attribute]
pub fn rsmemgen_impl_inline(_args :TokenStream , input :TokenStream) -> TokenStream {
    let mut implstruct : syn::ItemImpl ;

    match std::env::var("RSMEMCHK_MODE") {
        Ok(_v) => {},
        Err(_e) => {
            /*we do not set this handle*/
            rsmemgen_log_trace!("retv\n{}",input.to_string());
            return input;
        },
    }


    match syn::parse::<syn::ItemImpl>(input.clone()) {
        Ok(v) => {
            implstruct = v.clone();
        },
        Err(_e) => {
            syn_error_fmt!("not parse \n{}",input.to_string());
        }
    }

    let mut idx :usize = 0;
    while idx < implstruct.items.len() {
        let item = implstruct.items[idx].clone();
        let mut tk : TokenStream;
        let mut newfn :Option<syn::ImplItemFn> = None;

        match item {
            syn::ImplItem::Fn(fnptr) => {                
                tk = fnptr.clone().into_token_stream().into();
                rsmemgen_log_trace!("fnptr\n{}", tk.to_string());
                let mut inlinematch :bool = false;

                if fnptr.attrs.len() > 0 {
                    for a in fnptr.attrs.iter() {
                        tk = a.into_token_stream().into();
                        rsmemgen_log_trace!("a\n{}",tk.to_string());
                        match a.meta.clone() {
                            syn::Meta::List(l) => {
                                //tk = l.into_token_stream().into();
                                //rsmemgen_log_trace!("l\n{}",tk.to_string());
                                tk = l.path.into_token_stream().into();
                                rsmemgen_log_trace!("path\n{}",tk.to_string());
                                let pname = tk.to_string();
                                if pname == "inline" {
                                    inlinematch = true;
                                }
                            },
                            _ => {
                            },
                        }
                    }
                }
                if !inlinematch {
                    newfn = Some(fnptr.clone());
                    let  c :&mut syn::ImplItemFn = newfn.as_mut().unwrap();
                    let cp :syn::Attribute = syn::parse_quote! {
                         #[inline(never)]
                    };
                    c.attrs.push(cp.clone());
                }
            },
            _ => {},

        }

        if newfn.is_some() {
            implstruct.items[idx] = syn::ImplItem::Fn(newfn.as_ref().unwrap().clone());
        }
        idx += 1;
    }

    let retv :TokenStream = implstruct.into_token_stream().into();
    rsmemgen_log_trace!("retv\n{}",retv.to_string());

    return retv;
}

#[allow(unused_assignments)]
#[proc_macro_attribute]
pub fn rsmemgen_func_inline(_args :TokenStream , input :TokenStream) -> TokenStream {
    let mut fnstruct : syn::ItemFn ;

    match std::env::var("RSMEMCHK_MODE") {
        Ok(_v) => {},
        Err(_e) => {
            /*we do not set this handle*/
            rsmemgen_log_trace!("retv\n{}",input.to_string());
            return input;
        },
    }


    match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(v) => {
            fnstruct = v.clone();
        },
        Err(_e) => {
            syn_error_fmt!("not parse \n{}",input.to_string());
        }
    }

    let mut matched :bool = false;
    let mut idx :usize = 0;
    while idx < fnstruct.attrs.len() {
        let a :syn::Attribute = fnstruct.attrs[idx].clone();
        let mut tk : TokenStream;
        tk = a.clone().into_token_stream().into();
        rsmemgen_log_trace!("a\n{}",tk.to_string());
        match a.meta.clone() {
            syn::Meta::List(l) => {
                //tk = l.into_token_stream().into();
                //rsmemgen_log_trace!("l\n{}",tk.to_string());
                tk = l.path.into_token_stream().into();
                rsmemgen_log_trace!("path\n{}",tk.to_string());
                let pname = tk.to_string();
                if pname == "inline" {
                    matched = true;
                    break;
                }
            },
            _ => {
            },
        }
        idx += 1;
    }

    if !matched {
        let cp :syn::Attribute = syn::parse_quote! {
            #[inline(never)]
        };
        fnstruct.attrs.push(cp.clone());
    }


    let retv :TokenStream = fnstruct.into_token_stream().into();
    rsmemgen_log_trace!("retv\n{}",retv.to_string());
    return retv;
}
