use crate::cil::*;

pub fn analyze_arcs(blocks: &mut [CilBlock]) {
    let mut arcs: Vec<(CilLabel, CilLabel)> = Vec::new();

    for block in blocks.iter() {
        let src = block.label;
        for inst in &block.insts {
            match inst {
                CilInst::Branch(_, dst) => arcs.push((src, *dst)),
                _ => {},
            }
        }
    }

    dbg!(&arcs);

    for (src, dst) in arcs {
        blocks[src.index()].outgoing.push(dst);
        blocks[dst.index()].incoming.push(src);
    }
}

pub fn analyze_method(_file: &CilFile, method: &CilMethod) {
    if let Some(body) = method.body.write().unwrap().as_mut() {
        analyze_arcs(&mut body.blocks);
    }
}

pub fn analyze_assembly(file: CilFile) -> CilFile {
    for method in &file.methods {
        analyze_method(&file, method);
    }
    file
}
