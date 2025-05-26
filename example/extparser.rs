#[allow(unused_imports)]
use extargsparse_codegen::{extargs_load_commandline,ArgSet,extargs_map_function};
#[allow(unused_imports)]
use extargsparse_worker::{extargs_error_class,extargs_new_error};
#[allow(unused_imports)]
use extargsparse_worker::namespace::{NameSpaceEx};
#[allow(unused_imports)]
use extargsparse_worker::options::{ExtArgsOptions};
#[allow(unused_imports)]
use extargsparse_worker::argset::{ArgSetImpl};
use extargsparse_worker::parser::{ExtArgsParser};
use extargsparse_worker::funccall::{ExtArgsParseFunc};
#[allow(unused_imports)]
use extargsparse_worker::const_value::{COMMAND_SET,SUB_COMMAND_JSON_SET,COMMAND_JSON_SET,ENVIRONMENT_SET,ENV_SUB_COMMAND_JSON_SET,ENV_COMMAND_JSON_SET,DEFAULT_SET};
use extargsparse_worker::key::{KEYWORD_SUBCOMMAND};


#[allow(unused_imports)]
use std::cell::RefCell;
#[allow(unused_imports)]
use std::sync::Arc;
#[allow(unused_imports)]
use std::error::Error;
use std::boxed::Box;
#[allow(unused_imports)]
use regex::Regex;
#[allow(unused_imports)]
use std::any::Any;
use lazy_static::lazy_static;
use std::collections::HashMap;

use rsmalloc::{StackCallAllocEx,MemoryInfo};



#[global_allocator]
static ALLOCATOR: StackCallAllocEx = StackCallAllocEx{memsize : 307};


extargs_error_class!{ExtParserError}

fn loadcfg_handler(_ns :NameSpaceEx,_optargset :Option<Arc<RefCell<dyn ArgSetImpl>>>,_ctx :Option<Arc<RefCell<dyn Any>>>) -> Result<(),Box<dyn Error>> {
	Ok(())
}

#[extargs_map_function(loadcfg_handler)]
fn main() -> Result<(),Box<dyn Error>> {
	let parser :ExtArgsParser = ExtArgsParser::new(None,None)?;
	let commandline = format!(r#"
	{{

		"port|p" : 3990,
		"config|C" : null,
		"devconf" : null,
		"output|o" : null,
		"input|i" : null,
		"foreground|F" : false,
		"loadcfg<loadcfg_handler>##to test load handler##" : {{
			"$" : "*"
		}}
	}}
	"#);
	extargs_load_commandline!(parser,&commandline)?;
	let ores = parser.parse_commandline_ex(None,None,None,None);
	if ores.is_err() {
		let e = ores.err().unwrap();
		eprintln!("{:?}", e);
		return Err(e);
	}
	let ns :NameSpaceEx = ores.unwrap();
	if ns.get_string(KEYWORD_SUBCOMMAND).len() == 0 {
		eprintln!("no subcommand match");
		extargs_new_error!{ExtParserError,"no subcommand match"}
	}

	drop(ns);
	drop(parser);

	ALLOCATOR.scan();
	let _maps :MemoryInfo = ALLOCATOR.get_memory_info()?;
	return Ok(());
}
