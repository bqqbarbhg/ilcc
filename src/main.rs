use std::path::Path;

use ilcc::parsers::dotscope::parse_assembly_from_file;

fn main() {
    let path = std::env::args().nth(1).expect("expected a path");
    let assembly = parse_assembly_from_file(Path::new(&path))
        .expect("expected to parse assembly");

    for method in assembly.methods {
        println!("== {}", method.name);
        if let Some(body) = &method.body {
            for block in &body.blocks {
                println!("{}:", block.label);
                for inst in &block.insts {
                    println!("  {}", inst);
                }
            }
        }
    }
}
