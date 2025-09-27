use std::path::Path;

use ilcc::{analysis::analyze_assembly, parsers::dotscope::parse_assembly_from_file};

fn main() {
    let path = std::env::args().nth(1).expect("expected a path");
    let file = parse_assembly_from_file(Path::new(&path)).expect("expected to parse assembly");
    let file = analyze_assembly(file);

    for method in &file.methods {
        println!("== {}", method.name);
        if let Some(body) = &method.body.read().unwrap().as_ref() {
            for (ix, arg) in method.params.iter().enumerate() {
                println!("arg {}: {} {}", ix, arg.name, file.type_info(arg.typ).name);
            }
            for (ix, loc) in body.locals.iter().enumerate() {
                println!("loc {}: {}", ix, file.type_info(loc.typ).name);
            }

            for block in &body.blocks {
                let ins = block
                    .incoming
                    .iter()
                    .map(|src| {
                        let up = body.blocks[src.index()].offset > block.offset;
                        let ch = if up { "^" } else { "v" };
                        format!("{ch} {src}")
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("{}: ({ins})", block.label);
                for inst in &block.insts {
                    println!("  {}", inst);
                }
            }
        }
    }
}
