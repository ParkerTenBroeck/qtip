use proc_macros::{Diagnostic, Subdiagnostic};

use crate::{lex::Token, node::Node};

#[derive(Diagnostic)]
#[diag("expected symbol found {found}")]
pub struct ExpectedSymbol<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("expected type found {found}")]
pub struct ExpectedType<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("expected `;`, found {found}")]
pub struct ExpectedSemi<'a> {
    #[primary_node]
    #[label("help: add `;` here")]
    pub node: Node,

    pub found: Token<'a>,
    #[label("unexpected token")]
    pub found_node: Node,
}

#[derive(Diagnostic)]
#[diag("expected item, found {found}")]
#[help(
    "items can start with `pub`, `mod`, `union`, `struct`, `enum`, `static`, `const`, `mod`, `fn`"
)]
pub struct ExpectedItem<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,

    #[suggestion("remove this semicolon", code = "")]
    pub remove_semi: Option<Node>,
}

#[derive(Diagnostic)]
#[diag("expected expression, found {found}")]
pub struct ExpectedExpression<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("lambda expression bodies cannot have an explicit return type")]
pub struct LambdaExprBodyCannotHaveReturnType {
    #[primary_node]
    pub node: Node,
    #[subdiagnostic]
    pub wrap_in_block: WrapExprInBraces,
}

#[derive(Subdiagnostic)]
#[multipart_suggestion("wrap this expression body in braces")]
pub struct WrapExprInBraces {
    #[suggestion_part(code = "{{ ")]
    pub open: Node,
    #[suggestion_part(code = " }}")]
    pub close: Node,
}

#[derive(Diagnostic)]
#[diag("expected 'if', `for`, `loop`, `while`, `{{`, found {found}")]
pub struct ExpectedLabeledExpression<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("unexpected closing delimiter: {delim}")]
pub struct UnexpectedClosingDelim<'a> {
    #[primary_node]
    pub node: Node,
    pub delim: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("mismatched delimiters")]
pub struct MismatchedDelims {
    #[primary_node]
    pub lhs: Node,
    #[primary_node]
    pub rhs: Node,
}

#[derive(Diagnostic)]
#[diag("unclosed delimiter{$if unclosed.len() > 1{\"s\"}else{\"\"} }")]
pub struct UnclosedDelimiters {
    #[primary_node]
    pub node: Node,
    #[label("unclosed delimiter")]
    pub unclosed: Vec<Node>,
}

#[derive(Diagnostic)]
#[diag("unexpected token found {found} expected {expected:#}")]
pub struct UnexpectedToken<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,
    pub expected: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("missing lambda captures")]
pub struct MissingLambdaCaptures {
    #[primary_node]
    #[suggestion("add lambda captures", code = "[]")]
    pub node: Node,
}

#[derive(Diagnostic)]
#[diag("missing parameter list")]
pub struct MissingFnTypeParamList {
    #[primary_node]
    #[suggestion("add a parameter list", code = "()")]
    pub node: Node,
}

#[derive(Diagnostic)]
#[diag("missing block")]
pub struct MissingBlock {
    #[primary_node]
    #[suggestion("add block", code = "{{}}")]
    pub node: Node,
}

#[derive(Diagnostic)]
#[diag("missing parameter list")]
pub struct MissingFnParamList {
    #[primary_node]
    #[suggestion("add a parameter list", code = "()")]
    pub node: Node,
}

#[derive(Diagnostic)]
#[diag("missing function body")]
pub struct MissingFnBody {
    #[primary_node]
    #[suggestion("add a function body", code = "{{}}")]
    pub node: Node,
}

#[derive(Diagnostic)]
#[diag("incorrect delimiters used")]
pub struct IncorrectDelimiters {
    #[primary_node]
    pub lhs: Node,
    #[primary_node]
    pub rhs: Node,
    #[subdiagnostic]
    pub fix: DelimiterFix,
}

#[derive(Subdiagnostic)]
#[multipart_suggestion("replace {replace} with {replacement}")]
pub struct DelimiterFix {
    pub replace: &'static str,
    pub replacement: &'static str,
    #[suggestion_part(code = "{lhs_delim}")]
    pub lhs: Node,
    pub lhs_delim: &'static str,
    #[suggestion_part(code = "{rhs_delim}")]
    pub rhs: Node,
    pub rhs_delim: &'static str,
}

#[derive(Diagnostic)]
#[diag("Comments not tracked", level = "warning")]
pub struct CommentWarning {
    #[primary_node]
    pub node: Node,
}
