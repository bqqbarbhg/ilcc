use crate::cil::*;
use crate::parsers::disasm::disassemble;
use anyhow::{Context, Result, anyhow};
use dotscope;
use dotscope::prelude as ds;
use std::path::Path;

struct Parser {}

impl Parser {
    fn new() -> Parser {
        Parser {}
    }

    fn parse_body(&self, obj: &ds::CilObject, method: &ds::Method, body: &ds::MethodBody) -> Result<Option<CilFunc>> {
        let file = obj.file();

        println!("\n{}\n", method.name);
        for block in method.blocks() {
            println!("{}:", block.0);
            for inst in &block.1.instructions {
                println!("{}", inst.mnemonic);
            }
        }
        println!("");

        if let Some(rva) = method.rva {
            let method_offset = file.rva_to_offset(rva as usize)?;
            let code_offset = method_offset
                .checked_add(body.size_header)
                .ok_or_else(|| anyhow!("bad size header"))?;

            let blocks = disassemble(file.data(), code_offset)
                .with_context(|| format!("failed to parse CIL for {}()", method.name))?;

            let func = CilFunc { blocks };

            Ok(Some(func))
        } else {
            Ok(None)
        }
    }

    fn parse_method(&self, obj: &ds::CilObject, _name: ds::Token, method: &ds::Method) -> Result<CilMethod> {
        let body = match method.body.get() {
            Some(body) => self.parse_body(obj, method, body)?,
            None => None,
        };

        let method = CilMethod {
            name: method.name.clone(),
            body,
        };

        Ok(method)
    }

    fn parse_assembly(&self, obj: &ds::CilObject) -> Result<CilAssembly> {
        let methods = obj
            .methods()
            .iter()
            .map(|m| {
                self.parse_method(obj, *m.key(), m.value())
                    .with_context(|| format!("in method {}", m.value().name))
            })
            .collect::<Result<Vec<_>>>()?;

        let assembly = CilAssembly { methods };

        Ok(assembly)
    }
}

pub fn parse_assembly_from_file(path: &Path) -> Result<CilAssembly> {
    let obj = ds::CilObject::from_file(path)?;
    Parser::new().parse_assembly(&obj)
}
