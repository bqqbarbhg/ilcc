use std::fmt::Display;

macro_rules! index_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(std::num::NonZeroU32);

        impl $name {
            pub fn new(index: usize) -> Self {
                Self(std::num::NonZeroU32::new((index + 1) as u32).unwrap())
            }
            pub fn index(&self) -> usize {
                (self.0.get() - 1) as usize
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum CilSlot {
    Arg(u16),
    Loc(u16),
}

pub use CilSlot::Arg as CilArg;
pub use CilSlot::Loc as CilLoc;

impl Display for CilSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CilArg(arg) => write!(f, "arg.{}", arg),
            CilLoc(arg) => write!(f, "loc.{}", arg),
        }
    }
}

index_type!(CilLabel);

impl Display for CilLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "label.{}", self.index())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CilArith {
    Add,
    Mul,
    Sub,
    Div,
    Mod,
}

impl Display for CilArith {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = match self {
            CilArith::Add => "add",
            CilArith::Mul => "mul",
            CilArith::Sub => "sub",
            CilArith::Div => "div",
            CilArith::Mod => "mod",
        };
        write!(f, "{}", w)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CilCond {
    Always,
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
    NeUn,
    GeUn,
    GtUn,
    LeUn,
    LtUn,
    False,
    True,
}

impl Display for CilCond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = match self {
            CilCond::Always => "always",
            CilCond::Eq => "eq",
            CilCond::Ge => "ge",
            CilCond::Gt => "gt",
            CilCond::Le => "le",
            CilCond::Lt => "lt",
            CilCond::NeUn => "ne.un",
            CilCond::GeUn => "ge.un",
            CilCond::GtUn => "gt.un",
            CilCond::LeUn => "le.un",
            CilCond::LtUn => "lt.un",
            CilCond::False => "false",
            CilCond::True => "true",
        };
        write!(f, "{}", w)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CilTokenKind {
    Unknown(u8),
    Assembly,
    MethodDef,
    MethodSpec,
    MemberRef,
}

impl Display for CilTokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = match self {
            CilTokenKind::Unknown(v) => { return write!(f, "unk_{:02x}", v) },
            CilTokenKind::Assembly => "assembly",
            CilTokenKind::MethodDef => "methoddef",
            CilTokenKind::MethodSpec => "methodspec",
            CilTokenKind::MemberRef => "memberref",
        };
        write!(f, "{}", w)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CilToken(u32);

impl CilToken {
    pub fn new(value: u32) -> CilToken {
        CilToken(value)
    }

    pub fn token(&self) -> u32 {
        self.0
    }

    pub fn kind(&self) -> CilTokenKind {
        match self.0 >> 24 {
            0x06 => CilTokenKind::MethodDef,
            0x0a => CilTokenKind::MemberRef,
            0x20 => CilTokenKind::Assembly,
            0x2b => CilTokenKind::MethodSpec,
            v => CilTokenKind::Unknown(v as u8),
        }
    }

    pub fn index(&self) -> u32 {
        self.0 & 0xff_ffff
    }
}

impl Display for CilToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.kind(), self.index())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CilConst {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Display for CilConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CilConst::I32(v) => write!(f, "i32:{}", v),
            CilConst::I64(v) => write!(f, "i64:{}", v),
            CilConst::F32(v) => write!(f, "f32:{}", v),
            CilConst::F64(v) => write!(f, "f64:{}", v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CilPrefix {
    Unaligned,
    Volatile,
}

impl Display for CilPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CilPrefix::Unaligned => write!(f, "unaligned"),
            CilPrefix::Volatile => write!(f, "volatile"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CilInst {
    Nop,
    Break,
    Load(CilSlot),
    Addr(CilSlot),
    Store(CilSlot),
    Arith(CilArith),
    Const(CilConst),
    Call(CilToken),
    Compare(CilCond),
    Branch(CilCond, CilLabel),
    Prefix(CilPrefix),
    Return,
}

impl Display for CilInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CilInst::Nop => write!(f, "nop"),
            CilInst::Break => write!(f, "break"),
            CilInst::Load(slot) => write!(f, "load {}", slot),
            CilInst::Addr(slot) => write!(f, "addr {}", slot),
            CilInst::Store(slot) => write!(f, "store {}", slot),
            CilInst::Arith(op) => write!(f, "arith.{}", op),
            CilInst::Const(c) => write!(f, "const {}", c),
            CilInst::Call(token) => write!(f, "call {}", token),
            CilInst::Compare(cond) => write!(f, "compare.{}", cond),
            CilInst::Branch(cond, label) => write!(f, "branch.{} {}", cond, label),
            CilInst::Prefix(prefix) => write!(f, "prefix.{}", prefix),
            CilInst::Return => write!(f, "return"),
        }
    }
}

pub struct CilBlock {
    pub label: CilLabel,
    pub offset: usize,
    pub insts: Vec<CilInst>,
}

pub struct CilFunc {
    pub blocks: Vec<CilBlock>,
}

pub struct CilMethod {
    pub name: String,
    pub body: Option<CilFunc>,
}

pub struct CilAssembly {
    pub methods: Vec<CilMethod>,
}
