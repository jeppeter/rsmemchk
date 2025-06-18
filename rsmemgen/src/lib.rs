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

            //     match item {
            //         syn::ImplItem::Fn(fnptr) => {
            //             newfn = fnptr.clone();
            //             tk = fnptr.into_token_stream().into();
            //             rsmemgen_log_trace!("fnptr\n{}", tk.to_string());
            //             let mut inlinematch :bool = false;

            //             if fnptr.attrs.len() > 0 {
            //                 for a in fnptr.attrs.iter() {
            //                     tk = a.into_token_stream().into();
            //                     rsmemgen_log_trace!("a\n{}",tk.to_string());
            //                     tk = a.meta.clone().into_token_stream().into();
            //                     match a.meta.clone() {
            //                         syn::Meta::Path(p) => {
            //                     //tk = p.into_token_stream().into();
            //                     //rsmemgen_log_trace!("p\n{}",tk.to_string());
            //                 },
            //                 syn::Meta::List(l) => {
            //                     //tk = l.into_token_stream().into();
            //                     //rsmemgen_log_trace!("l\n{}",tk.to_string());
            //                     tk = l.path.into_token_stream().into();
            //                     rsmemgen_log_trace!("path\n{}",tk.to_string());
            //                     let pname = tk.to_string();
            //                     if pname == "inline" {
            //                         inlinematch = true;
            //                     }
            //                 },
            //                 syn::Meta::NameValue(n)=>{
            //                     tk = n.into_token_stream().into();
            //                     rsmemgen_log_trace!("n\n{}",tk.to_string());                                
            //                 },
            //             }
            //                 if !inlinematch {
            //                     let  c :&mut syn::ImplItemFn = newfn.as_mut().unwrap();
            //                     let cp :syn::Attribute = syn::parse_quote! {
            //                        #[inline(never)]
            //                     };
            //                     c.attrs.push(cp.clone());
            //                 }
            //             }

            //         }
            //     },
            // _ => {},
