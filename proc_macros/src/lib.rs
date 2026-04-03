use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{Span as ProcSpan, TokenStream as TokenStream2};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, GenericArgument,
    Ident, LitStr, PathArguments, Result, Token, Type, Variant,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_derive(
    Diagnostic,
    attributes(
        diag,
        note,
        label,
        primary_node,
        primary_span,
        suggestion,
        subdiagnostic
    )
)]
pub fn derive_diagnostic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_diagnostic(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_diagnostic(input: DeriveInput) -> Result<TokenStream2> {
    match input.data.clone() {
        Data::Struct(data) => expand_struct(input, data),
        Data::Enum(data) => expand_enum(input, data),
        Data::Union(_) => Err(Error::new(
            input.span(),
            "Diagnostic cannot be derived for unions",
        )),
    }
}

fn expand_struct(input: DeriveInput, data: DataStruct) -> Result<TokenStream2> {
    let (pattern, handlers) = build_handlers(input.span(), &input.attrs, &data.fields, true)?;

    let name = input.ident;
    let generics = input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics Diagnostic for #name #ty_generics #where_clause {
            #[allow(unused)]
            fn to_diag<'a>(self, ctx: &Context<'a>) -> Diag<'a> {
                let #pattern = self;
                // let mut diag = Vec::new();

                #( #handlers )*

                // diag

                vec![group]
            }
        }
    })
}

fn expand_enum(input: DeriveInput, data: DataEnum) -> Result<TokenStream2> {
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut arms = Vec::new();
    for variant in data.variants {
        arms.push(expand_variant_arm(&variant)?);
    }

    Ok(quote! {
        impl #impl_generics Diagnostic for #name #ty_generics #where_clause {
            #[allow(unused)]
            fn to_diag<'a>(self, ctx: &Context<'a>) -> Group<'a> {
                match self {
                    #( #arms ),*
                }
            }
        }
    })
}

fn expand_variant_arm(variant: &Variant) -> Result<TokenStream2> {
    let vname = &variant.ident;

    let (pattern, handlers) =
        build_handlers(variant.span(), &variant.attrs, &variant.fields, false)?;

    Ok(quote! {
        Self::#vname #pattern => {
            // let mut diag = Vec::new();

            #( #handlers )*

            // diag
            vec![group]
        }
    })
}

struct DiagAttr {
    level: Level,
    title: LitStr,
}

impl Parse for DiagAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            level: Level::new(LevelKind::Error, input.span()),
            title: input.parse::<LitStr>()?,
        })
    }
}

struct HelpAttr {
    msg: LitStr,
}

impl Parse for HelpAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            msg: input.parse::<LitStr>()?,
        })
    }
}

enum LevelKind {
    Help,
    Info,
    Warning,
    Error,
    Note,
}

struct Level {
    span: ProcSpan,
    kind: LevelKind,
}

impl Level {
    pub fn new(kind: LevelKind, span: ProcSpan) -> Self {
        Self { span, kind }
    }
}

impl ToTokens for Level {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let str = match self.kind {
            LevelKind::Help => "HELP",
            LevelKind::Info => "INFO",
            LevelKind::Warning => "WARNING",
            LevelKind::Error => "ERROR",
            LevelKind::Note => "NOTE",
        };
        tokens.append(Ident::new(str, self.span));
    }
}

fn build_handlers(
    span: proc_macro2::Span,
    attrs: &[Attribute],
    fields: &Fields,
    is_struct: bool,
) -> Result<(TokenStream2, Vec<TokenStream2>)> {
    let mut handlers = vec![];
    let built = build_field_patterns(fields, is_struct)?;

    fn help_note(
        field: Option<&Field>,
        attr: &Attribute,
        kind: Level,
        built: &FieldPatterns,
    ) -> Result<TokenStream2> {
        let HelpAttr { msg } = attr.parse_args()?;
        if let Some(field) = field {
            let Some(ident) = built.field_map.get(field) else {
                return Err(Error::new(field.span(), "improperly constructed field map"));
            };
            let ty = &field.ty;
            if is_type_named(ty, "bool") {
                Ok(quote! {
                    if #ident {
                        group = group.element(Level::#kind.message(format!(#msg)));
                    }
                })
            } else if is_type_named(ty, "Option") || is_type_named(ty, "Vec") {
                Ok(quote! {
                    for node in &#ident{
                        let src = ctx.sources.get_idx(node.src).unwrap();
                        let snippet = Snippet::source(&src.contents)
                            .path(src.path.as_os_str().to_str().unwrap());

                        group = group.element(Level::#kind.primary_title(format!(#msg))
                            .element(
                                snippet.
                            )
                            .annotation(AnnotationKind::Primary.span(node.range.into())));   
                    }
                })
            } else if is_type_named(ty, "Node") {
                Ok(quote! {
                    let src = ctx.sources.get_idx(#ident.src).unwrap();
                    let snippet = Snippet::source(&src.contents)
                        .path(src.path.as_os_str().to_str().unwrap());

                    group = group.element(Level::#kind.primary_title(format!(#msg))
                        .annotation(AnnotationKind::Primary.span(node.range.into())));
                })
            } else {
                Err(Error::new(
                    ty.span(),
                    format!(
                        "{} defined on fields must be a bool, Span, Option<Span>, or Vec<Span>",
                        attr.path().get_ident().unwrap()
                    ),
                ))
            }
        } else {
            Ok(quote! {
                group = group.element(Level::#kind.message(format!(#msg)));
            })
        }
    }

    for attr in attrs {
        if attr.path().is_ident("diag") {
            let DiagAttr { level, title } = attr.parse_args()?;
            handlers.push(quote! {
                let mut group = Group::with_title(Level::#level.primary_title(format!(#title)));
            });
        } else if attr.path().is_ident("multipart_suggestion") {
            todo!()
            // let DiagAttr { level, title } = attr.parse_args()?;
            // handlers.push(quote! {
            //     let group = Group::with_title(Level::HELP.primary_title(format!(#title)));
            // });
        } else if attr.path().is_ident("note") {
            let level = Level::new(LevelKind::Note, attr.path().span());
            handlers.push(help_note(None, attr, level, &built)?);
        } else if attr.path().is_ident("help") {
            let level = Level::new(LevelKind::Help, attr.path().span());
            handlers.push(help_note(None, attr, level, &built)?);
        }
    }

    for field in fields {
        for attr in &field.attrs {
            if attr.path().is_ident("primary_span") {
            } else if attr.path().is_ident("label") {
            } else if attr.path().is_ident("note") {
                let level = Level::new(LevelKind::Note, attr.path().span());
                handlers.push(help_note(Some(field), attr, level, &built)?);
            } else if attr.path().is_ident("help") {
                let level = Level::new(LevelKind::Help, attr.path().span());
                handlers.push(help_note(Some(field), attr, level, &built)?);
            }
        }
    }

    Ok((built.pattern, handlers))
}

struct FieldPatterns {
    pattern: TokenStream2,
    field_map: HashMap<Field, Ident>,
}

fn build_field_patterns(fields: &Fields, is_struct: bool) -> Result<FieldPatterns> {
    match fields {
        Fields::Named(fields) => build_named_fields(fields.named.iter(), is_struct),
        Fields::Unnamed(fields) => build_unnamed_fields(fields.unnamed.iter(), is_struct),
        Fields::Unit => Ok(FieldPatterns {
            pattern: quote!(),
            field_map: HashMap::new(),
        }),
    }
}

fn build_named_fields<'a>(
    fields: impl Iterator<Item = &'a Field>,
    is_struct: bool,
) -> Result<FieldPatterns> {
    let mut bindings = Vec::new();
    let mut field_map = HashMap::new();

    for field in fields {
        let binding = field
            .ident
            .clone()
            .ok_or_else(|| Error::new(field.span(), "expected named field"))?;

        field_map.insert(field.clone(), binding.clone());
        bindings.push(binding.clone());
    }

    let pattern = if is_struct {
        quote! { Self { #( #bindings ),* } }
    } else {
        quote! { { #( #bindings ),* } }
    };

    Ok(FieldPatterns { pattern, field_map })
}

fn build_unnamed_fields<'a>(
    fields: impl Iterator<Item = &'a Field>,
    is_struct: bool,
) -> Result<FieldPatterns> {
    let mut bindings = Vec::new();
    let mut field_map = HashMap::new();

    for (idx, field) in fields.enumerate() {
        let binding = Ident::new(&format!("__field_{idx}"), ProcSpan::call_site());

        field_map.insert(field.clone(), binding.clone());
        bindings.push(binding.clone());
    }

    let pattern = if is_struct {
        quote! { Self( #( #bindings ),* ) }
    } else {
        quote! { ( #( #bindings ),* ) }
    };

    Ok(FieldPatterns { pattern, field_map })
}

fn expand_field_handlers(field: &Field, binding: &Ident, ty: &Type) -> Result<Vec<TokenStream2>> {
    let mut out = Vec::new();

    let mut seen_primary_node = false;
    let mut seen_primary_span = false;
    let mut seen_subdiagnostic = false;

    for attr in &field.attrs {
        if attr.path().is_ident("primary_node") {
            if seen_primary_node {
                return Err(Error::new(attr.span(), "duplicate #[primary_node]"));
            }
            seen_primary_node = true;
            out.push(expand_primary_node_handler(binding, ty));
        } else if attr.path().is_ident("primary_span") {
            if seen_primary_span {
                return Err(Error::new(attr.span(), "duplicate #[primary_span]"));
            }
            seen_primary_span = true;
            out.push(expand_primary_span_handler(binding, ty));
        } else if attr.path().is_ident("subdiagnostic") {
            if seen_subdiagnostic {
                return Err(Error::new(attr.span(), "duplicate #[subdiagnostic]"));
            }
            seen_subdiagnostic = true;
            out.push(expand_subdiagnostic_handler(binding, ty));
        } else if attr.path().is_ident("suggestion") {
            let suggestion = attr.parse_args::<SuggestionAttr>()?;
            out.push(expand_suggestion_handler(binding, ty, suggestion)?);
        } else if attr.path().is_ident("diag") || attr.path().is_ident("note") {
            return Err(Error::new(
                attr.span(),
                "field-level #[diag(...)] and #[note(...)] are not supported",
            ));
        }
    }

    Ok(out)
}

fn expand_primary_node_handler(binding: &Ident, ty: &Type) -> TokenStream2 {
    if is_type_named(ty, "Node") {
        quote! {
            {
                let src = ctx.sources.get_idx(#binding.src).unwrap();
                let snippet = Snippet::source(&src.contents)
                    .path(src.path.as_os_str().to_str().unwrap())
                    .annotation(AnnotationKind::Primary.span(#binding.range.into()));
                diag = diag.element(snippet);
            }
        }
    } else if is_option_of_named(ty, "Node") {
        quote! {
            if let Some(node) = #binding {
                let src = ctx.sources.get_idx(node.src).unwrap();
                let snippet = Snippet::source(&src.contents)
                    .path(src.path.as_os_str().to_str().unwrap())
                    .annotation(AnnotationKind::Primary.span(node.range.into()));
                diag = diag.element(snippet);
            }
        }
    } else {
        quote! {
            ::core::compile_error!("#[primary_node] is only supported on Node or Option<Node>");
        }
    }
}

fn expand_primary_span_handler(binding: &Ident, ty: &Type) -> TokenStream2 {
    if is_type_named(ty, "Span") {
        quote! {
            {
                diag = diag.element(AnnotationKind::Primary.span(#binding.into()));
            }
        }
    } else if is_option_of_named(ty, "Span") {
        quote! {
            if let Some(span) = #binding {
                diag = diag.element(AnnotationKind::Primary.span(span.into()));
            }
        }
    } else {
        quote! {
            ::core::compile_error!("#[primary_span] is only supported on Span or Option<Span>");
        }
    }
}

fn expand_subdiagnostic_handler(binding: &Ident, ty: &Type) -> TokenStream2 {
    if is_option_type(ty) {
        quote! {
            if let Some(subdiag) = #binding {
                diag = Subdiagnostic::add_to_diag(subdiag, diag, ctx);
            }
        }
    } else {
        quote! {
            diag = Subdiagnostic::add_to_diag(#binding, diag, ctx);
        }
    }
}

fn expand_suggestion_handler(
    binding: &Ident,
    ty: &Type,
    suggestion: SuggestionAttr,
) -> Result<TokenStream2> {
    let message = rewrite_template(&suggestion.message);
    let code = rewrite_template(&suggestion.code);
    let style = expand_suggestion_style(&suggestion.style)?;
    let applicability = expand_applicability(&suggestion.applicability)?;

    if is_type_named(ty, "Span") {
        Ok(quote! {
            {
                let suggestion = Suggestion::new()
                    .span(#binding.into())
                    .message(format!(#message))
                    .style(#style)
                    .applicability(#applicability)
                    .code(format!(#code));
                diag = diag.element(suggestion);
            }
        })
    } else if is_option_of_named(ty, "Span") {
        Ok(quote! {
            if let Some(span) = #binding {
                let suggestion = Suggestion::new()
                    .span(span.into())
                    .message(format!(#message))
                    .style(#style)
                    .applicability(#applicability)
                    .code(format!(#code));
                diag = diag.element(suggestion);
            }
        })
    } else {
        Ok(quote! {
            ::core::compile_error!("#[suggestion(...)] is only supported on Span or Option<Span> fields");
        })
    }
}

fn expand_suggestion_style(style: &LitStr) -> Result<TokenStream2> {
    match style.value().as_str() {
        "short" => Ok(quote!(SuggestionStyle::Short)),
        "verbose" => Ok(quote!(SuggestionStyle::Verbose)),
        other => Err(Error::new(
            style.span(),
            format!("unknown suggestion style `{other}`"),
        )),
    }
}

fn expand_applicability(applicability: &LitStr) -> Result<TokenStream2> {
    match applicability.value().as_str() {
        "machine-applicable" => Ok(quote!(Applicability::MachineApplicable)),
        "maybe-incorrect" => Ok(quote!(Applicability::MaybeIncorrect)),
        "has-placeholders" => Ok(quote!(Applicability::HasPlaceholders)),
        "unspecified" => Ok(quote!(Applicability::Unspecified)),
        other => Err(Error::new(
            applicability.span(),
            format!("unknown applicability `{other}`"),
        )),
    }
}

fn find_unique_lit_str_attr(attrs: &[Attribute], name: &str) -> Result<Option<LitStr>> {
    let mut out = None;

    for attr in attrs {
        if attr.path().is_ident(name) {
            let lit = attr
                .parse_args::<LitStr>()
                .map_err(|_| Error::new(attr.span(), format!("expected #[{}(\"...\")]", name)))?;

            if out.is_some() {
                return Err(Error::new(
                    attr.span(),
                    format!("duplicate #[{}(...)] attribute", name),
                ));
            }

            out = Some(lit);
        }
    }

    Ok(out)
}

fn find_all_lit_str_attrs(attrs: &[Attribute], name: &str) -> Result<Vec<LitStr>> {
    let mut out = Vec::new();

    for attr in attrs {
        if attr.path().is_ident(name) {
            let lit = attr
                .parse_args::<LitStr>()
                .map_err(|_| Error::new(attr.span(), format!("expected #[{}(\"...\")]", name)))?;
            out.push(lit);
        }
    }

    Ok(out)
}

fn rewrite_template(lit: &LitStr) -> LitStr {
    let s = lit.value();
    let rewritten = rewrite_template_str(&s);
    LitStr::new(&rewritten, lit.span())
}

fn rewrite_template_str(s: &str) -> String {
    // Rewrites `{$ident}` -> `{ident}`
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && matches!(chars.peek(), Some('$')) {
            let _ = chars.next();
            out.push('{');
        } else {
            out.push(ch);
        }
    }

    out
}

fn is_type_named(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(tp) => tp.path.segments.last().is_some_and(|seg| seg.ident == name),
        _ => false,
    }
}

fn is_option_of_named(ty: &Type, inner_name: &str) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };

    let Some(seg) = tp.path.segments.last() else {
        return false;
    };

    if seg.ident != "Option" {
        return false;
    }

    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };

    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return false;
    };

    is_type_named(inner, inner_name)
}

fn is_option_type(ty: &Type) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };

    tp.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "Option")
}

struct SuggestionAttr {
    message: LitStr,
    style: LitStr,
    applicability: LitStr,
    code: LitStr,
}

impl Parse for SuggestionAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let message: LitStr = input.parse()?;

        let mut style = None;
        let mut applicability = None;
        let mut code = None;

        while !input.is_empty() {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "style" => style = Some(value),
                "applicability" => applicability = Some(value),
                "code" => code = Some(value),
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unknown suggestion key `{other}`"),
                    ));
                }
            }
        }

        Ok(Self {
            style: style.ok_or_else(|| {
                Error::new(
                    message.span(),
                    "missing `style = \"...\"` in #[suggestion(...)]",
                )
            })?,
            code: code.ok_or_else(|| {
                Error::new(
                    message.span(),
                    "missing `code = \"...\"` in #[suggestion(...)]",
                )
            })?,
            applicability: applicability.ok_or_else(|| {
                Error::new(
                    message.span(),
                    "missing `applicability = \"...\"` in #[suggestion(...)]",
                )
            })?,
            message,
        })
    }
}
