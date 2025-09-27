use std::collections::BTreeMap;

use anyhow::{Result, anyhow, bail};

use crate::cil::*;

#[inline]
fn arg_n<const N: usize>(inst: &[u8], pos: &mut usize) -> Result<[u8; N]> {
    let p = *pos;
    let slice = inst
        .get(p..p + N)
        .ok_or_else(|| anyhow!("instruction argument out of bounds"))?;

    *pos += N;
    Ok(unsafe { *(slice.as_ptr() as *const [u8; N]) })
}

#[inline]
fn arg_u8(inst: &[u8], pos: &mut usize) -> Result<u8> {
    Ok(u8::from_le_bytes(arg_n::<1>(inst, pos)?))
}

#[inline]
fn arg_u16(inst: &[u8], pos: &mut usize) -> Result<u16> {
    Ok(u16::from_le_bytes(arg_n::<2>(inst, pos)?))
}

#[inline]
fn arg_u32(inst: &[u8], pos: &mut usize) -> Result<u32> {
    Ok(u32::from_le_bytes(arg_n::<4>(inst, pos)?))
}

#[inline]
fn arg_i8(inst: &[u8], pos: &mut usize) -> Result<i8> {
    Ok(i8::from_le_bytes(arg_n::<1>(inst, pos)?))
}

#[inline]
fn arg_i16(inst: &[u8], pos: &mut usize) -> Result<i16> {
    Ok(i16::from_le_bytes(arg_n::<2>(inst, pos)?))
}

#[inline]
fn arg_i32(inst: &[u8], pos: &mut usize) -> Result<i32> {
    Ok(i32::from_le_bytes(arg_n::<4>(inst, pos)?))
}

#[inline]
fn arg_i64(inst: &[u8], pos: &mut usize) -> Result<i64> {
    Ok(i64::from_le_bytes(arg_n::<8>(inst, pos)?))
}

#[inline]
fn arg_f32(inst: &[u8], pos: &mut usize) -> Result<f32> {
    Ok(f32::from_le_bytes(arg_n::<4>(inst, pos)?))
}

#[inline]
fn arg_f64(inst: &[u8], pos: &mut usize) -> Result<f64> {
    Ok(f64::from_le_bytes(arg_n::<8>(inst, pos)?))
}

#[derive(Debug, Clone)]
pub enum DecodeInst {
    Nop,
    Break,
    Load(CilSlot),
    Store(CilSlot),
    Addr(CilSlot),
    Arith(CilArith),
    Const(CilConst),
    Compare(CilCond),
    Branch(CilCond, i32),
    Return,
    Call(CilToken),
    Prefix(CilPrefix),
}

pub fn decode(image: &[u8], pos: usize) -> Result<(DecodeInst, usize)> {
    let op = image.get(pos).ok_or_else(|| anyhow!("instruction out of bounds"))?;

    let mut pos = pos + 1;
    let p = &mut pos;

    let inst = match op {
        0x00 => DecodeInst::Nop,
        0x01 => DecodeInst::Break,
        0x02 => DecodeInst::Load(CilArg(0)),
        0x03 => DecodeInst::Load(CilArg(1)),
        0x04 => DecodeInst::Load(CilArg(2)),
        0x05 => DecodeInst::Load(CilArg(3)),
        0x06 => DecodeInst::Load(CilLoc(0)),
        0x07 => DecodeInst::Load(CilLoc(1)),
        0x08 => DecodeInst::Load(CilLoc(2)),
        0x09 => DecodeInst::Load(CilLoc(3)),
        0x0a => DecodeInst::Store(CilLoc(0)),
        0x0b => DecodeInst::Store(CilLoc(1)),
        0x0c => DecodeInst::Store(CilLoc(2)),
        0x0d => DecodeInst::Store(CilLoc(3)),
        0x0e => DecodeInst::Load(CilArg(arg_u8(image, p)? as u16)),
        0x0f => DecodeInst::Addr(CilArg(arg_u8(image, p)? as u16)),
        0x10 => DecodeInst::Store(CilArg(arg_u8(image, p)? as u16)),
        0x15 => DecodeInst::Const(CilConst::I32(-1)),
        0x16 => DecodeInst::Const(CilConst::I32(0)),
        0x17 => DecodeInst::Const(CilConst::I32(1)),
        0x18 => DecodeInst::Const(CilConst::I32(2)),
        0x19 => DecodeInst::Const(CilConst::I32(3)),
        0x1a => DecodeInst::Const(CilConst::I32(4)),
        0x1b => DecodeInst::Const(CilConst::I32(5)),
        0x1c => DecodeInst::Const(CilConst::I32(6)),
        0x1d => DecodeInst::Const(CilConst::I32(7)),
        0x1e => DecodeInst::Const(CilConst::I32(8)),
        0x1f => DecodeInst::Const(CilConst::I32(arg_i8(image, p)? as i32)),
        0x20 => DecodeInst::Const(CilConst::I32(arg_i32(image, p)?)),
        0x21 => DecodeInst::Const(CilConst::I64(arg_i64(image, p)?)),
        0x22 => DecodeInst::Const(CilConst::F32(arg_f32(image, p)?)),
        0x23 => DecodeInst::Const(CilConst::F64(arg_f64(image, p)?)),
        0x28 => DecodeInst::Call(CilToken::new(arg_u32(image, p)?)),
        0x2a => DecodeInst::Return,
        0x2b => DecodeInst::Branch(CilCond::Always, arg_i8(image, p)? as i32),
        0x2c => DecodeInst::Branch(CilCond::False, arg_i8(image, p)? as i32),
        0x2d => DecodeInst::Branch(CilCond::True, arg_i8(image, p)? as i32),
        0x2e => DecodeInst::Branch(CilCond::Eq, arg_i8(image, p)? as i32),
        0x2f => DecodeInst::Branch(CilCond::Ge, arg_i8(image, p)? as i32),
        0x30 => DecodeInst::Branch(CilCond::Gt, arg_i8(image, p)? as i32),
        0x31 => DecodeInst::Branch(CilCond::Le, arg_i8(image, p)? as i32),
        0x32 => DecodeInst::Branch(CilCond::Lt, arg_i8(image, p)? as i32),
        0x33 => DecodeInst::Branch(CilCond::NeUn, arg_i8(image, p)? as i32),
        0x34 => DecodeInst::Branch(CilCond::GeUn, arg_i8(image, p)? as i32),
        0x35 => DecodeInst::Branch(CilCond::GtUn, arg_i8(image, p)? as i32),
        0x36 => DecodeInst::Branch(CilCond::LeUn, arg_i8(image, p)? as i32),
        0x37 => DecodeInst::Branch(CilCond::LtUn, arg_i8(image, p)? as i32),
        0x38 => DecodeInst::Branch(CilCond::Always, arg_i32(image, p)?),
        0x39 => DecodeInst::Branch(CilCond::False, arg_i32(image, p)?),
        0x3a => DecodeInst::Branch(CilCond::True, arg_i32(image, p)?),
        0x3b => DecodeInst::Branch(CilCond::Eq, arg_i32(image, p)?),
        0x3c => DecodeInst::Branch(CilCond::Ge, arg_i32(image, p)?),
        0x3d => DecodeInst::Branch(CilCond::Gt, arg_i32(image, p)?),
        0x3e => DecodeInst::Branch(CilCond::Le, arg_i32(image, p)?),
        0x3f => DecodeInst::Branch(CilCond::Lt, arg_i32(image, p)?),
        0x40 => DecodeInst::Branch(CilCond::NeUn, arg_i32(image, p)?),
        0x41 => DecodeInst::Branch(CilCond::GeUn, arg_i32(image, p)?),
        0x42 => DecodeInst::Branch(CilCond::GtUn, arg_i32(image, p)?),
        0x43 => DecodeInst::Branch(CilCond::LeUn, arg_i32(image, p)?),
        0x44 => DecodeInst::Branch(CilCond::LtUn, arg_i32(image, p)?),
        0x58 => DecodeInst::Arith(CilArith::Add),
        0x59 => DecodeInst::Arith(CilArith::Sub),
        0x5a => DecodeInst::Arith(CilArith::Mul),
        0x5b => DecodeInst::Arith(CilArith::Div),
        0xfe => match arg_u8(image, p)? {
            0x01 => DecodeInst::Compare(CilCond::Eq),
            0x02 => DecodeInst::Compare(CilCond::Gt),
            0x03 => DecodeInst::Compare(CilCond::GtUn),
            0x04 => DecodeInst::Compare(CilCond::Lt),
            0x05 => DecodeInst::Compare(CilCond::LtUn),
            0x09 => DecodeInst::Load(CilArg(arg_u16(image, p)?)),
            0x0a => DecodeInst::Addr(CilArg(arg_u16(image, p)?)),
            0x0b => DecodeInst::Store(CilArg(arg_u16(image, p)?)),
            0x0c => DecodeInst::Load(CilLoc(arg_u16(image, p)?)),
            0x0d => DecodeInst::Addr(CilLoc(arg_u16(image, p)?)),
            0x0e => DecodeInst::Store(CilLoc(arg_u16(image, p)?)),
            0x12 => DecodeInst::Prefix(CilPrefix::Unaligned),
            unk => bail!("unknown CIL instruction {:02x} {:02X}", 0xfe, unk),
        },
        unk => bail!("unkonwn CIL instrucion {:02X}", unk),
    };

    Ok((inst, pos))
}

struct DecodeBlock {
    insts: Vec<(usize, CilInst)>,
    begin: usize,
    end: usize,
    finished: bool,
}

impl DecodeBlock {
    fn finish(&self, label: CilLabel) -> CilBlock {
        let insts = self.insts.iter().map(|(_, inst)| *inst).collect();
        CilBlock {
            insts,
            label,
            offset: self.begin,
        }
    }

    fn split(&mut self, pos: usize, new_label: CilLabel) -> Result<DecodeBlock> {
        let split = self
            .insts
            .binary_search_by_key(&pos, |(off, _)| *off)
            .map_err(|_| anyhow!("bad branch target"))?;

        let new_insts = &self.insts[split..];
        let new_block = DecodeBlock {
            insts: new_insts.to_vec(),
            begin: pos,
            end: self.end,
            finished: true,
        };

        self.insts.truncate(split);
        self.insts.push((self.end, CilInst::Branch(CilCond::Always, new_label)));

        self.end = pos;

        Ok(new_block)
    }
}

fn find_block(targets: &BTreeMap<usize, CilLabel>, blocks: &[DecodeBlock], offset: usize) -> Option<CilLabel> {
    let mut before = targets.range(..offset);
    let Some((_, &label)) = before.next_back() else {
        return None;
    };

    let block = &blocks[label.index()];
    if offset < block.end { Some(label) } else { None }
}

pub fn disassemble(image: &[u8], pos: usize) -> Result<Vec<CilBlock>> {
    let mut targets: BTreeMap<usize, CilLabel> = BTreeMap::new();
    let mut blocks: Vec<DecodeBlock> = Vec::new();
    let mut work: Vec<CilLabel> = Vec::new();

    let init_label = CilLabel::new(blocks.len());
    targets.insert(pos, init_label);
    blocks.push(DecodeBlock {
        insts: Vec::new(),
        begin: pos,
        end: pos,
        finished: false,
    });
    work.push(init_label);

    let base = pos;
    while let Some(cur_label) = work.pop() {
        println!("continuing {}", cur_label);
        loop {
            let pos = {
                let block = &blocks[cur_label.index()];
                if block.finished {
                    println!("finished {}", cur_label);
                    break;
                }
                block.end
            };

            println!("decoding at {:+#x}", (pos as isize - base as isize));

            if let Some(label) = find_block(&targets, &blocks, pos + 1) {
                println!("ran into block {}, finishing {}", label, cur_label);
                let block = &blocks[label.index()];
                if pos == block.begin && block.end > block.begin {
                    // Ran into a following block that is being worked on
                    let block = &mut blocks[cur_label.index()];
                    block.insts.push((block.end, CilInst::Branch(CilCond::Always, label)));
                    block.finished = true;
                    break;
                } else {
                    // Ran into the middle of a block..
                    bail!("bad instruction stream");
                }
            }

            let (inst, end) = decode(image, pos)?;
            println!("decoded {:?}, now at {:+#x}", inst, (end as isize - base as isize));

            let inst = match inst {
                DecodeInst::Nop => CilInst::Nop,
                DecodeInst::Break => CilInst::Break,
                DecodeInst::Load(slot) => CilInst::Load(slot),
                DecodeInst::Addr(slot) => CilInst::Addr(slot),
                DecodeInst::Store(slot) => CilInst::Store(slot),
                DecodeInst::Arith(arith) => CilInst::Arith(arith),
                DecodeInst::Const(cst) => CilInst::Const(cst),
                DecodeInst::Call(token) => CilInst::Call(token),
                DecodeInst::Prefix(prefix) => CilInst::Prefix(prefix),
                DecodeInst::Compare(cond) => CilInst::Compare(cond),
                DecodeInst::Branch(cond, offset) => {
                    let target = (end as isize + offset as isize) as usize;
                    println!("branch to {:+#x} ({:#x})", (target as isize - base as isize), target);
                    println!("{:#x?}", targets);
                    let (self_label, target_label) = if let Some(&tb) = targets.get(&target) {
                        // Jump to the beginning of a given block
                        println!("simple branch to {}", tb);
                        (cur_label, tb)
                    } else {
                        if let Some(tb) = find_block(&targets, &blocks, target) {
                            // Jump to the middle of a block
                            let label = CilLabel::new(blocks.len());
                            println!("middle of {}, splitting to: {}", tb, label);

                            let target_block = &mut blocks[tb.index()];
                            let new_block = target_block.split(target, label)?;
                            let self_label = if tb == cur_label { label } else { cur_label };

                            targets.insert(target, label);
                            blocks.push(new_block);
                            work.push(label);

                            (self_label, label)
                        } else {
                            // Jump to a new block
                            let label = CilLabel::new(blocks.len());
                            let target_block = DecodeBlock {
                                insts: Vec::new(),
                                begin: target,
                                end: target,
                                finished: false,
                            };

                            println!("new block: {}", label);

                            targets.insert(target, label);
                            blocks.push(target_block);
                            work.push(label);

                            (cur_label, label)
                        }
                    };

                    let block = &mut blocks[self_label.index()];
                    block.insts.push((pos, CilInst::Branch(cond, target_label)));
                    block.end = end;
                    block.finished = true;

                    match cond {
                        CilCond::Always => {
                            println!("unconditional branch, finishing");
                        }
                        _ => {
                            if targets.get(&end).is_none() {
                                let cont_label = CilLabel::new(blocks.len());

                                println!("making continuation branch: {}", cont_label);
                                let block = &mut blocks[self_label.index()];
                                block.insts.push((end, CilInst::Branch(CilCond::Always, cont_label)));

                                let cont_block = DecodeBlock {
                                    insts: Vec::new(),
                                    begin: end,
                                    end: end,
                                    finished: false,
                                };
                                targets.insert(end, cont_label);
                                blocks.push(cont_block);
                                work.push(cont_label);
                            }
                        }
                    }

                    break;
                }
                DecodeInst::Return => {
                    println!("return: finishing");
                    let block = &mut blocks[cur_label.index()];
                    block.insts.push((pos, CilInst::Return));
                    block.end = end;
                    block.finished = true;
                    break;
                }
            };

            println!("simple: {}", inst);

            let block = &mut blocks[cur_label.index()];
            block.insts.push((pos, inst));
            block.end = end;
        }
    }

    let blocks = blocks
        .iter()
        .enumerate()
        .map(|(ix, block)| block.finish(CilLabel::new(ix)))
        .collect();

    Ok(blocks)
}
