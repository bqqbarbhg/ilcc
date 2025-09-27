use crate::cil::*;
use crate::parsers::disasm::disassemble;
use anyhow::{Context, Result, anyhow};
use dotscope;
use dotscope::prelude as ds;
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

#[derive(Default)]
struct Parser {
    types: Vec<CilTypeInfo>,
    type_map: HashMap<u32, CilType>,
}

impl Parser {
    fn new() -> Parser {
        Parser { ..Default::default() }
    }

    fn init_type(&mut self, typ: &ds::CilTypeRef) -> Result<CilTypeInfo> {
        let ti = CilTypeInfo {
            name: typ.name().unwrap(),
        };

        Ok(ti)
    }

    fn get_type(&mut self, typ: &ds::CilTypeRef) -> Result<CilType> {
        let token = typ.token().unwrap();
        if let Some(typ) = self.type_map.get(&token.value()) {
            return Ok(*typ);
        }

        let new_ti = self
            .init_type(typ)
            .with_context(|| format!("failed to initialize type '{}'", typ.name().unwrap_or_else(String::new)))?;

        let new_typ = CilType::new(self.types.len());
        self.types.push(new_ti);
        self.type_map.insert(token.value(), new_typ);

        Ok(new_typ)
    }

    fn parse_body(
        &mut self,
        obj: &ds::CilObject,
        method: &ds::Method,
        body: &ds::MethodBody,
    ) -> Result<Option<CilFunc>> {
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

            let mut locals = Vec::new();

            for (_, local) in method.local_vars.iter() {
                let typ = &local.base;
                let typ = self.get_type(typ)?;

                locals.push(CilLocal { name: None, typ });
            }

            let func = CilFunc { blocks, locals };

            Ok(Some(func))
        } else {
            Ok(None)
        }
    }

    fn parse_method(&mut self, obj: &ds::CilObject, _name: ds::Token, method: &ds::Method) -> Result<CilMethod> {
        let body = match method.body.get() {
            Some(body) => self.parse_body(obj, method, body)?,
            None => None,
        };

        let mut params = Vec::new();
        for (_, arg) in method.params.iter() {
            let typ = arg.base.get().unwrap();
            let typ = self.get_type(typ)?;

            params.push(CilParam {
                name: arg.name.clone().unwrap(),
                typ,
            });
        }

        let method = CilMethod {
            name: method.name.clone(),
            body: RwLock::new(body),
            params,
        };

        Ok(method)
    }

    fn parse_assembly(&mut self, obj: &ds::CilObject) -> Result<CilFile> {
        let methods = obj
            .methods()
            .iter()
            .map(|m| {
                self.parse_method(obj, *m.key(), m.value())
                    .with_context(|| format!("in method {}", m.value().name))
            })
            .collect::<Result<Vec<_>>>()?;

        let assembly = CilFile {
            methods,
            types: std::mem::take(&mut self.types),
        };

        Ok(assembly)
    }
}

pub fn parse_assembly_from_file(path: &Path) -> Result<CilFile> {
    let obj = ds::CilObject::from_file(path)?;
    Parser::new().parse_assembly(&obj)
}
