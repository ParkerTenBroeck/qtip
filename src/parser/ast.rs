use super::Node;
use crate::lex::{Number, StringLiteral};

#[derive(Debug)]
pub struct Program<'a>(pub Vec<Item<'a>>);

#[derive(Debug)]
pub enum ItemKind<'a> {
    Module(),
    Use(),
    Fn(Fn<'a>),
    Extern(),
    Static(),
    Constant(),
    Struct(),
    Enum(),
    Union(),
}

#[derive(Debug)]
pub struct Item<'a> {
    pub node: Node,
    pub kind: ItemKind<'a>,
    pub vis: Vis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vis {
    Pub,
    Priv,
}

#[derive(Debug)]
pub struct Symbol<'a> {
    pub name: &'a str,
    pub node: Node,
}

#[derive(Debug)]
pub struct Fn<'a> {
    pub name: Symbol<'a>,
    pub params: Vec<FnParam<'a>>,
    pub ret: Option<Type<'a>>,
    pub body: Option<Expr<'a>>,
}

#[derive(Debug)]
pub struct FnParam<'a> {
    pub node: Node,
    pub vis: Vis,
    pub name: Symbol<'a>,
    pub ty: Type<'a>,
}

#[derive(Debug)]
pub struct Type<'a> {
    pub name: Symbol<'a>,
}

#[derive(Debug)]
pub struct Let<'a> {
    pub name: Symbol<'a>,
    pub ty: Type<'a>,
    pub initializer: Option<Expr<'a>>,
}

#[derive(Debug)]
pub struct Stmt<'a> {
    pub node: Node,
    pub kind: StmtKind<'a>,
}

#[derive(Debug)]
pub enum StmtKind<'a> {
    Item(Item<'a>),
    Expr(Expr<'a>),
    Let(Let<'a>),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    And,
    Xor,
    Or,
    Shl,
    Shr,

    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Ne,

    Assign,
    PlusAssign,
    ModuloAssign,
    DivideAssign,
    TimesAssign,
    MinusAssign,
    OrAssign,
    AndAssign,
    XorAssign,
    ShiftRightAssign,
    ShiftLeftAssign,
}

impl BinOp {
    pub fn precedence(&self) -> u32 {
        match self {
            BinOp::Add => 20 - 4,
            BinOp::Sub => 20 - 4,

            BinOp::Mul => 20 - 3,
            BinOp::Div => 20 - 3,
            BinOp::Rem => 20 - 3,

            BinOp::Shl => 20 - 5,
            BinOp::Shr => 20 - 5,

            BinOp::Lt => 20 - 6,
            BinOp::Lte => 20 - 6,
            BinOp::Gt => 20 - 6,
            BinOp::Gte => 20 - 6,

            BinOp::Eq => 20 - 7,
            BinOp::Ne => 20 - 7,

            BinOp::And => 20 - 8,
            BinOp::Xor => 20 - 9,
            BinOp::Or => 20 - 10,

            BinOp::Assign => 20 - 14,
            BinOp::PlusAssign => 20 - 14,
            BinOp::ModuloAssign => 20 - 14,
            BinOp::DivideAssign => 20 - 14,
            BinOp::TimesAssign => 20 - 14,
            BinOp::MinusAssign => 20 - 14,
            BinOp::OrAssign => 20 - 14,
            BinOp::AndAssign => 20 - 14,
            BinOp::XorAssign => 20 - 14,
            BinOp::ShiftRightAssign => 20 - 14,
            BinOp::ShiftLeftAssign => 20 - 14,
        }
    }

    pub fn right_to_left(&self) -> bool {
        match self {
            BinOp::Add => false,
            BinOp::Sub => false,
            BinOp::Mul => false,
            BinOp::Div => false,
            BinOp::Rem => false,
            BinOp::Shl => false,
            BinOp::Shr => false,
            BinOp::Lt => false,
            BinOp::Lte => false,
            BinOp::Gt => false,
            BinOp::Gte => false,
            BinOp::Eq => false,
            BinOp::Ne => false,
            BinOp::And => false,
            BinOp::Xor => false,
            BinOp::Or => false,

            BinOp::Assign => true,
            BinOp::PlusAssign => true,
            BinOp::ModuloAssign => true,
            BinOp::DivideAssign => true,
            BinOp::TimesAssign => true,
            BinOp::MinusAssign => true,
            BinOp::OrAssign => true,
            BinOp::AndAssign => true,
            BinOp::XorAssign => true,
            BinOp::ShiftRightAssign => true,
            BinOp::ShiftLeftAssign => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub struct CondBlock<'a> {
    pub cond: Expr<'a>,
    pub block: Expr<'a>,
}

#[derive(Debug)]
pub struct Block<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

#[derive(Debug)]
pub struct Expr<'a> {
    pub node: Node,
    pub kind: ExprKind<'a>,
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    Block(Block<'a>),
    If {
        if_chain: Vec<CondBlock<'a>>,
        else_end: Option<Box<Expr<'a>>>,
    },
    While(Box<CondBlock<'a>>),
    Loop(Box<Expr<'a>>),
    FuncCall {
        ptr: Box<Expr<'a>>,
        args: Vec<Expr<'a>>,
    },
    BinOp {
        lhs: Box<Expr<'a>>,
        op: BinOp,
        rhs: Box<Expr<'a>>,
    },
    UnOp {
        expr: Box<Expr<'a>>,
        op: UnOp,
    },
    Cast {
        expr: Box<Expr<'a>>,
        ty: Type<'a>,
    },
    Path(Symbol<'a>),
    Literal(Literal<'a>),
    Paren(Box<Expr<'a>>),
}

#[derive(Debug)]
pub enum Literal<'a> {
    Number(Number<'a>),
    String(StringLiteral<'a>),
    Char(StringLiteral<'a>),
    Bool(bool),
}
