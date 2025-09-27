use std::path::Path;

use ilcc::parsers::dotscope::parse_assembly_from_file;

fn main() {
    let path = std::env::args().nth(1).expect("expected a path");
    let file = parse_assembly_from_file(Path::new(&path)).expect("expected to parse assembly");

    for method in &file.methods {
        println!("== {}", method.name);
        if let Some(body) = &method.body {
            for (ix, arg) in method.params.iter().enumerate() {
                println!("arg {}: {} {}", ix, arg.name, file.type_info(arg.typ).name);
            }
            for (ix, loc) in body.locals.iter().enumerate() {
                println!("loc {}: {}", ix, file.type_info(loc.typ).name);
            }

            for block in &body.blocks {
                println!("{}:", block.label);
                for inst in &block.insts {
                    println!("  {}", inst);
                }
            }
        }
    }
}
