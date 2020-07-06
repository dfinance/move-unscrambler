use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::access::ModuleAccess;
use libra::vm::access::ScriptAccess;
use libra::vm::errors::BinaryLoaderResult;
use crate::deps::map::ModInfo;
use crate::disasm::*;
use crate::types::*;
use super::mod_addr::ExtractModAddr;
use super::*;


type FnHandleAddrs = Vec<FnAddr>;

impl<T> Extract<FnHandleAddrs> for T where T: MoveAccess {
	fn extract(&self) -> FnHandleAddrs {
		self.function_handles()
		    .iter()
		    .map(|f| {
			    let module = {
				    let mh = self.module_handle_at(f.module);
				    let name = self.identifier_at(mh.name);
				    let address = self.address_identifier_at(mh.address);
				    ModAddr::new(address.to_owned(), name)
			    };
			    let fn_addr: FnAddr = (module, self.identifier_at(f.name)).into();
			    //  let ret = self.signature_at(f.return_);
			    // parameters
			    // type_parameters
			    fn_addr
		    })
		    .collect()
	}
}


pub fn extract_fn_handles<'a>(bc: CompiledMoveRef<'a>) -> Vec<FnAddr> { bc.extract() }
