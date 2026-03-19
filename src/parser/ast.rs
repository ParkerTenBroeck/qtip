
pub enum Item<'a>{
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

pub enum Vis{
    Pub,
    Priv,
}


pub struct Func<'a>{
    pub vis: Vis,
    pub name: &'a str,
    pub args: Vec<FuncParam<'a>>
}

pub struct FuncParam<'a>{
    pub vis: Vis,
    pub name: &'a str,
    pub ty: Type<'a>,
}

pub struct Type<'a>{
    pub name: &'a str,
}

pub enum Stmt<'a>{
    Item(Box<Item<'a>>),
    Expr(Expr<'a>),
    Let{
        name: &'a str,
        ty: Type<'a>,
        initializer: Option<Expr<'a>>,
    }
}

pub enum BinOp{
    Add,
    Sub,
    Mul,
    Div,

}

pub enum UnOp{
    Neg,
    Not,
    Ref,
    DeRef,
}


pub enum Expr<'a>{
    Block(Vec<Stmt<'a>>),
    If{
        if_: Vec<(Expr<'a>, Expr<'a>)>,
        else_: Option<Box<(Expr<'a>, Expr<'a>)>>,
    },
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
    Name(&'a str)
}