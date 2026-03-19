use crate::lex::Span;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node{
    pub range: Span,
}

#[derive(Debug)]
pub struct Program<'a>(pub Vec<Item<'a>>);

#[derive(Debug)]
pub enum ItemKind<'a>{
    Module(),
    Use(),
    Function(Func<'a>),
    Extern(),
    Static(),
    Constant(),
    Struct(),
    Enum(),
    Union(),
}

#[derive(Debug)]
pub struct Item<'a>{
    pub node: Node,
    pub kind: ItemKind<'a>,
    pub vis: Vis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vis{
    Pub,
    Priv,
}

#[derive(Debug)]
pub struct Symbol<'a>{
    pub name: &'a str,
    pub node: Node,
}

#[derive(Debug)]
pub struct Func<'a>{
    pub name: Symbol<'a>,
    pub args: Vec<FuncParam<'a>>
}

#[derive(Debug)]
pub struct FuncParam<'a>{
    pub node: Node,
    pub vis: Vis,
    pub name: Symbol<'a>,
    pub ty: Type<'a>,
}

#[derive(Debug)]
pub struct Type<'a>{
    pub name: Symbol<'a>,
}

#[derive(Debug)]
pub struct Let<'a>{
    pub name: Symbol<'a>,
    pub ty: Type<'a>,
    pub initializer: Option<Expr<'a>>,
}

#[derive(Debug)]
pub enum Stmt<'a>{
    Item(ItemKind<'a>),
    Expr(Expr<'a>),
    Let(Let<'a>)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp{
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp{
    Neg,
    Not,
    Ref,
    DeRef,
}

#[derive(Debug)]
pub struct CondBlock<'a>{
    pub cond: Expr<'a>,
    pub block: Expr<'a>,
}

#[derive(Debug)]
pub struct Expr<'a>{
    pub node: Node,
    pub kind: ExprKind<'a>,
}

#[derive(Debug)]
pub enum ExprKind<'a>{
    Block(Vec<Stmt<'a>>),
    If{
        if_chain: Vec<CondBlock<'a>>,
        else_end: Option<Box<Expr<'a>>>,
    },
    While(Box<CondBlock<'a>>),
    Loop(Box<Expr<'a>>),
    FuncCall{
        ptr: Box<Expr<'a>>,
        args: Vec<Expr<'a>>,
    },
    BinOp{
        lhs: Box<Expr<'a>>,
        op: BinOp,
        rhs: Box<Expr<'a>>,
    },
    UnOp{
        expr: Box<Expr<'a>>,
        op: UnOp,
    },
    Cast {
        expr: Box<Expr<'a>>,
        ty: Type<'a>,
    },
    Path(Symbol<'a>)
}