use proc_macro::TokenStream;
use proc_macro2::{Span as ProcSpan, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, GenericArgument,
    Expr, Ident, LitStr, PathArguments, Result, Token, Type, parse::Parse, parse::ParseStream,
    parse_macro_input, spanned::Spanned,
};

struct DiagAttr {
    title: LitStr,
    level: DiagnosticLevel,
}

impl Parse for DiagAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let title: LitStr = input.parse()?;
        let mut level = DiagnosticLevel::Error;

        while !input.is_empty() {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "level" => level = DiagnosticLevel::parse(&value)?,
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unknown #[diag(...)] key `{other}`"),
                    ));
                }
            }
        }

        Ok(Self {
            title,
            level,
        })
    }
}

enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Note,
    Help,
}

impl DiagnosticLevel {
    fn parse(lit: &LitStr) -> Result<Self> {
        match lit.value().as_str() {
            "error" => Ok(Self::Error),
            "warning" => Ok(Self::Warning),
            "info" => Ok(Self::Info),
            "note" => Ok(Self::Note),
            "help" => Ok(Self::Help),
            other => Err(Error::new(
                lit.span(),
                format!("unknown diagnostic level `{other}`"),
            )),
        }
    }

    fn tokens(&self) -> TokenStream2 {
        match self {
            Self::Error => quote!(ERROR),
            Self::Warning => quote!(WARNING),
            Self::Info => quote!(INFO),
            Self::Note => quote!(NOTE),
            Self::Help => quote!(HELP),
        }
    }
}

#[proc_macro_derive(
    Diagnostic,
    attributes(
        diag,
        help,
        note,
        label,
        multipart_suggestion,
        primary_node,
        suggestion,
        suggestion_part,
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

#[proc_macro_derive(
    Subdiagnostic,
    attributes(
        help,
        note,
        label,
        multipart_suggestion,
        primary_node,
        suggestion,
        suggestion_part
    )
)]
pub fn derive_subdiagnostic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_subdiagnostic(input) {
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

fn expand_subdiagnostic(input: DeriveInput) -> Result<TokenStream2> {
    match input.data.clone() {
        Data::Struct(data) => expand_subdiagnostic_struct(input, data),
        Data::Enum(data) => expand_subdiagnostic_enum(input, data),
        Data::Union(_) => Err(Error::new(
            input.span(),
            "Subdiagnostic cannot be derived for unions",
        )),
    }
}

fn expand_struct(input: DeriveInput, data: DataStruct) -> Result<TokenStream2> {
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = expand_body(&input.attrs, &data.fields, true)?;

    Ok(quote! {
        impl #impl_generics crate::diag::Diagnostic for #name #ty_generics #where_clause {
            fn to_diag<'a>(self, ctx: &crate::context::Context<'a>) -> crate::diag::Diag<'a> {
                #body
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
        let vname = variant.ident.clone();
        let body = expand_body(&variant.attrs, &variant.fields, false)?;
        let pattern = destructure_pattern(&variant.fields, false)?;

        arms.push(quote! {
            Self::#vname #pattern => {
                #body
            }
        });
    }

    Ok(quote! {
        impl #impl_generics crate::diag::Diagnostic for #name #ty_generics #where_clause {
            fn to_diag<'a>(self, ctx: &crate::context::Context<'a>) -> crate::diag::Diag<'a> {
                match self {
                    #( #arms ),*
                }
            }
        }
    })
}

fn expand_subdiagnostic_struct(input: DeriveInput, data: DataStruct) -> Result<TokenStream2> {
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let body = expand_subdiagnostic_body(&input.attrs, &data.fields, true)?;

    Ok(quote! {
        impl #impl_generics crate::diag::Subdiagnostic for #name #ty_generics #where_clause {
            fn add_to_diag<'a>(
                self,
                ctx: &crate::context::Context<'a>,
                group: &mut ::annotate_snippets::Group<'a>,
                groups: &mut crate::diag::Diag<'a>,
            ) {
                #body
            }
        }
    })
}

fn expand_subdiagnostic_enum(input: DeriveInput, data: DataEnum) -> Result<TokenStream2> {
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut arms = Vec::new();
    for variant in data.variants {
        let vname = variant.ident.clone();
        let body = expand_subdiagnostic_body(&variant.attrs, &variant.fields, false)?;
        let pattern = destructure_pattern(&variant.fields, false)?;

        arms.push(quote! {
            Self::#vname #pattern => {
                #body
            }
        });
    }

    Ok(quote! {
        impl #impl_generics crate::diag::Subdiagnostic for #name #ty_generics #where_clause {
            fn add_to_diag<'a>(
                self,
                ctx: &crate::context::Context<'a>,
                group: &mut ::annotate_snippets::Group<'a>,
                groups: &mut crate::diag::Diag<'a>,
            ) {
                match self {
                    #( #arms ),*
                }
            }
        }
    })
}

fn expand_body(attrs: &[Attribute], fields: &Fields, is_struct: bool) -> Result<TokenStream2> {
    let diag = find_diag_attr(attrs)?;
    let level = diag.level.tokens();
    let title = format_template(&diag.title)?;
    let group_messages = expand_group_messages(attrs)?;
    let field_handlers = expand_field_handlers(fields)?;
    let bind_self = if is_struct {
        let pattern = destructure_pattern(fields, true)?;
        Some(quote! { let #pattern = self; })
    } else {
        None
    };

    Ok(quote! {
        #bind_self
        let mut groups = Vec::new();
        let mut group = ::annotate_snippets::Group::with_title(
            ::annotate_snippets::Level::#level.primary_title(#title)
        );
        #( #field_handlers )*
        #( #group_messages )*
        groups.insert(0, group);
        groups
    })
}

fn expand_subdiagnostic_body(
    attrs: &[Attribute],
    fields: &Fields,
    is_struct: bool,
) -> Result<TokenStream2> {
    let kind = parse_subdiagnostic_kind(attrs)?;
    let fields_info = subdiag_fields(fields)?;
    let body = expand_subdiagnostic_apply(&kind, &fields_info)?;
    let bind_self = if is_struct {
        let pattern = destructure_pattern(fields, true)?;
        Some(quote! { let #pattern = self; })
    } else {
        None
    };

    Ok(quote! {
        #bind_self
        #body
    })
}

fn destructure_pattern(fields: &Fields, is_struct: bool) -> Result<TokenStream2> {
    match fields {
        Fields::Named(named) => {
            let bindings = named
                .named
                .iter()
                .map(|field| {
                    field
                        .ident
                        .clone()
                        .ok_or_else(|| Error::new(field.span(), "expected named field"))
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(if is_struct {
                quote! { Self { #( #bindings ),* } }
            } else {
                quote! { { #( #bindings ),* } }
            })
        }
        Fields::Unnamed(unnamed) => {
            let bindings = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| Ident::new(&format!("__field_{idx}"), ProcSpan::call_site()))
                .collect::<Vec<_>>();

            Ok(if is_struct {
                quote! { Self( #( #bindings ),* ) }
            } else {
                quote! { ( #( #bindings ),* ) }
            })
        }
        Fields::Unit => Ok(quote! {}),
    }
}

fn find_diag_attr(attrs: &[Attribute]) -> Result<DiagAttr> {
    let mut diag = None;

    for attr in attrs {
        if attr.path().is_ident("diag") {
            let parsed = attr.parse_args::<DiagAttr>().map_err(|_| {
                Error::new(
                    attr.span(),
                    "expected #[diag(\"...\")] or #[diag(\"...\", level = \"...\")]",
                )
            })?;

            if diag.is_some() {
                return Err(Error::new(attr.span(), "duplicate #[diag(...)] attribute"));
            }

            diag = Some(parsed);
        } else if attr.path().is_ident("suggestion") {
            return Err(Error::new(
                attr.span(),
                "this diagnostic derive does not support #[suggestion] yet",
            ));
        }
    }

    diag.ok_or_else(|| Error::new(ProcSpan::call_site(), "missing #[diag(\"...\")] attribute"))
}

fn expand_group_messages(attrs: &[Attribute]) -> Result<Vec<TokenStream2>> {
    let mut out = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("note") {
            let msg = parse_lit_attr(attr, "note")?;
            out.push(quote! {
                group = group.element(::annotate_snippets::Level::NOTE.message(#msg));
            });
        } else if attr.path().is_ident("help") {
            let msg = parse_lit_attr(attr, "help")?;
            out.push(quote! {
                group = group.element(::annotate_snippets::Level::HELP.message(#msg));
            });
        }
    }

    Ok(out)
}

fn expand_field_handlers(fields: &Fields) -> Result<Vec<TokenStream2>> {
    let mut handlers = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        let binding = field_binding(field, idx)?;
        let attrs = parse_field_attrs(field)?;

        if attrs.suggestion {
            return Err(Error::new(
                field.span(),
                "#[suggestion] is not supported yet",
            ));
        }

        if attrs.primary_node {
            handlers.push(expand_primary_node_handler(&binding, &field.ty, &attrs.labels)?);
        } else if !attrs.labels.is_empty() {
            handlers.push(expand_context_label_handler(&binding, &field.ty, &attrs.labels)?);
        }

        handlers.extend(expand_field_message_handlers(&binding, &field.ty, &attrs.notes)?);
        if attrs.subdiagnostic {
            handlers.push(expand_subdiagnostic_field_handler(&binding, &field.ty)?);
        }
    }

    Ok(handlers)
}

fn field_binding(field: &Field, idx: usize) -> Result<Ident> {
    Ok(match &field.ident {
        Some(ident) => ident.clone(),
        None => Ident::new(&format!("__field_{idx}"), ProcSpan::call_site()),
    })
}

fn expand_primary_node_handler(
    binding: &Ident,
    ty: &Type,
    labels: &[LitStr],
) -> Result<TokenStream2> {
    let annotation = primary_annotation_tokens(quote!(node.range.into()), labels);

    if is_type_named(ty, "Node") {
        Ok(quote! {
            {
                let node = #binding;
                let src = ctx.sources.get_idx(node.src).unwrap();
                let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                    .path(src.path.display().to_string())
                    #annotation;
                group = group.element(snippet);
            }
        })
    } else if is_option_of_named(ty, "Node") {
        Ok(quote! {
            if let Some(node) = #binding {
                let src = ctx.sources.get_idx(node.src).unwrap();
                let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                    .path(src.path.display().to_string())
                    #annotation;
                group = group.element(snippet);
            }
        })
    } else {
        Err(Error::new(
            ty.span(),
            "#[primary_node] is only supported on Node or Option<Node>",
        ))
    }
}

fn expand_context_label_handler(
    binding: &Ident,
    ty: &Type,
    labels: &[LitStr],
) -> Result<TokenStream2> {
    let annotation = context_annotation_tokens(quote!(node.range.into()), labels);
    let grouped_annotations = labels.iter().map(|label| {
        let label = format_template(label).unwrap();
        quote! {
            annotations.push((
                node,
                ::annotate_snippets::AnnotationKind::Context
                    .span(node.range.into())
                    .label(#label),
            ));
        }
    });

    if is_type_named(ty, "Node") {
        Ok(quote! {
            {
                let node = #binding;
                let src = ctx.sources.get_idx(node.src).unwrap();
                let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                    .path(src.path.display().to_string())
                    #annotation;
                group = group.element(snippet);
            }
        })
    } else if is_option_of_named(ty, "Node") {
        Ok(quote! {
            if let Some(node) = #binding {
                let src = ctx.sources.get_idx(node.src).unwrap();
                let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                    .path(src.path.display().to_string())
                    #annotation;
                group = group.element(snippet);
            }
        })
    } else if is_vec_of_named(ty, "Node") {
        Ok(quote! {
            let snippets = crate::diag::annotation_snippets(ctx, {
                let mut annotations = Vec::new();
                for node in #binding.iter().copied() {
                    #( #grouped_annotations )*
                }
                annotations
            });
            for snippet in snippets {
                group = group.element(snippet);
            }
        })
    } else {
        Err(Error::new(
            ty.span(),
            "#[label(...)] is only supported on Node, Option<Node>, or Vec<Node> fields",
        ))
    }
}

fn expand_field_message_handlers(
    binding: &Ident,
    ty: &Type,
    messages: &[FieldMessage],
) -> Result<Vec<TokenStream2>> {
    let mut out = Vec::new();

    for message in messages {
        let level = message.level.tokens();
        let msg = format_template(&message.message)?;

        if is_type_named(ty, "bool") {
            out.push(quote! {
                if #binding {
                    groups.push(::annotate_snippets::Group::with_title(
                        ::annotate_snippets::Level::#level.secondary_title(#msg)
                    ));
                }
            });
        } else if is_type_named(ty, "Node") {
            out.push(quote! {
                {
                    let node = #binding;
                    let src = ctx.sources.get_idx(node.src).unwrap();
                    let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                        .path(src.path.display().to_string())
                        .annotation(::annotate_snippets::AnnotationKind::Primary.span(node.range.into()));
                    groups.push(::annotate_snippets::Group::with_title(
                        ::annotate_snippets::Level::#level.secondary_title(#msg)
                    ).element(snippet));
                }
            });
        } else if is_option_of_named(ty, "Node") {
            out.push(quote! {
                if let Some(node) = #binding {
                    let src = ctx.sources.get_idx(node.src).unwrap();
                    let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                        .path(src.path.display().to_string())
                        .annotation(::annotate_snippets::AnnotationKind::Primary.span(node.range.into()));
                    groups.push(::annotate_snippets::Group::with_title(
                        ::annotate_snippets::Level::#level.secondary_title(#msg)
                    ).element(snippet));
                }
            });
        } else if is_vec_of_named(ty, "Node") {
            out.push(quote! {
                if !#binding.is_empty() {
                    let mut note_group = ::annotate_snippets::Group::with_title(
                        ::annotate_snippets::Level::#level.secondary_title(#msg)
                    );
                    let snippets = crate::diag::annotation_snippets(
                        ctx,
                        #binding.iter().copied().map(|node| {
                            (
                                node,
                                ::annotate_snippets::AnnotationKind::Primary.span(node.range.into()),
                            )
                        }),
                    );
                    for snippet in snippets {
                        note_group = note_group.element(snippet);
                    }
                    groups.push(note_group);
                }
            });
        } else {
            out.push(quote! {
                groups.push(::annotate_snippets::Group::with_title(
                    ::annotate_snippets::Level::#level.secondary_title(#msg)
                ));
            });
        }
    }

    Ok(out)
}

fn expand_subdiagnostic_field_handler(binding: &Ident, ty: &Type) -> Result<TokenStream2> {
    if is_option_type(ty) {
        Ok(quote! {
            if let Some(subdiag) = #binding {
                crate::diag::Subdiagnostic::add_to_diag(subdiag, ctx, &mut group, &mut groups);
            }
        })
    } else if is_vec_type(ty) {
        Ok(quote! {
            for subdiag in #binding {
                crate::diag::Subdiagnostic::add_to_diag(subdiag, ctx, &mut group, &mut groups);
            }
        })
    } else {
        Ok(quote! {
            crate::diag::Subdiagnostic::add_to_diag(#binding, ctx, &mut group, &mut groups);
        })
    }
}

fn parse_subdiagnostic_kind(attrs: &[Attribute]) -> Result<SubdiagnosticKind> {
    let mut out = None;

    for attr in attrs {
        let kind = if attr.path().is_ident("label") {
            Some(SubdiagnosticKind::Label(parse_lit_attr(attr, "label")?))
        } else if attr.path().is_ident("note") {
            Some(SubdiagnosticKind::Note(parse_lit_attr(attr, "note")?))
        } else if attr.path().is_ident("help") {
            Some(SubdiagnosticKind::Help(parse_lit_attr(attr, "help")?))
        } else if attr.path().is_ident("suggestion") {
            Some(SubdiagnosticKind::Suggestion(SuggestionKind::Single(
                attr.parse_args::<SuggestionAttr>()?,
            )))
        } else if attr.path().is_ident("multipart_suggestion") {
            Some(SubdiagnosticKind::Suggestion(SuggestionKind::Multipart(
                attr.parse_args::<MultipartSuggestionAttr>()?,
            )))
        } else {
            None
        };

        if let Some(kind) = kind {
            if out.is_some() {
                return Err(Error::new(
                    attr.span(),
                    "subdiagnostics must have exactly one primary attribute: #[label], #[note], #[help], #[suggestion], or #[multipart_suggestion]",
                ));
            }
            out = Some(kind);
        }
    }

    out.ok_or_else(|| {
        Error::new(
            ProcSpan::call_site(),
            "missing subdiagnostic kind; expected one of #[label], #[note], #[help], #[suggestion], or #[multipart_suggestion]",
        )
    })
}

fn subdiag_fields(fields: &Fields) -> Result<SubdiagFields> {
    let mut primary_node = None;
    let mut suggestion_parts = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        let binding = field_binding(field, idx)?;
        let attrs = parse_field_attrs(field)?;

        if attrs.suggestion || attrs.subdiagnostic {
            return Err(Error::new(
                field.span(),
                "nested #[suggestion] and #[subdiagnostic] are not supported in #[derive(Subdiagnostic)]",
            ));
        }

        if attrs.primary_node {
            if primary_node.is_some() {
                return Err(Error::new(field.span(), "duplicate #[primary_node]"));
            }
            primary_node = Some(SubdiagPrimaryNode {
                binding: binding.clone(),
                ty: field.ty.clone(),
            });
        }

        if let Some(part) = attrs.suggestion_part {
            suggestion_parts.push(SuggestionPartField {
                binding,
                ty: field.ty.clone(),
                code: part.code,
            });
        }
    }

    Ok(SubdiagFields {
        primary_node,
        suggestion_parts,
    })
}

fn expand_subdiagnostic_apply(
    kind: &SubdiagnosticKind,
    fields: &SubdiagFields,
) -> Result<TokenStream2> {
    match kind {
        SubdiagnosticKind::Label(message) => {
            let primary = fields.primary_node.as_ref().ok_or_else(|| {
                Error::new(
                    ProcSpan::call_site(),
                    "#[label(...)] subdiagnostics require a #[primary_node] field",
                )
            })?;

            let annotation = context_annotation_tokens(quote!(node.range.into()), std::slice::from_ref(message));
            expand_subdiagnostic_node_apply(primary, quote! {
                let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                    .path(src.path.display().to_string())
                    #annotation;
                crate::diag::append_to_group(group, snippet);
            })
        }
        SubdiagnosticKind::Note(message) => expand_secondary_subdiag_group(
            fields.primary_node.as_ref(),
            quote!(NOTE),
            message,
        ),
        SubdiagnosticKind::Help(message) => expand_secondary_subdiag_group(
            fields.primary_node.as_ref(),
            quote!(HELP),
            message,
        ),
        SubdiagnosticKind::Suggestion(suggestion) => {
            expand_suggestion_subdiagnostic(suggestion, fields)
        }
    }
}

fn expand_secondary_subdiag_group(
    primary: Option<&SubdiagPrimaryNode>,
    level: TokenStream2,
    message: &LitStr,
) -> Result<TokenStream2> {
    let message = format_template(message)?;

    match primary {
        Some(primary) => expand_subdiagnostic_node_apply(primary, quote! {
            let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                .path(src.path.display().to_string())
                .annotation(::annotate_snippets::AnnotationKind::Primary.span(node.range.into()));
            groups.push(
                ::annotate_snippets::Group::with_title(
                    ::annotate_snippets::Level::#level.secondary_title(#message)
                )
                .element(snippet)
            );
        }),
        None => Ok(quote! {
            groups.push(::annotate_snippets::Group::with_title(
                ::annotate_snippets::Level::#level.secondary_title(#message)
            ));
        }),
    }
}

fn expand_subdiagnostic_node_apply(
    primary: &SubdiagPrimaryNode,
    body: TokenStream2,
) -> Result<TokenStream2> {
    let binding = &primary.binding;
    let ty = &primary.ty;

    if is_type_named(ty, "Node") {
        Ok(quote! {
            {
                let node = #binding;
                let src = ctx.sources.get_idx(node.src).unwrap();
                #body
            }
        })
    } else if is_option_of_named(ty, "Node") {
        Ok(quote! {
            if let Some(node) = #binding {
                let src = ctx.sources.get_idx(node.src).unwrap();
                #body
            }
        })
    } else if is_vec_of_named(ty, "Node") {
        Ok(quote! {
            for node in #binding {
                let src = ctx.sources.get_idx(node.src).unwrap();
                #body
            }
        })
    } else {
        Err(Error::new(
            ty.span(),
            "#[primary_node] is only supported on Node, Option<Node>, or Vec<Node>",
        ))
    }
}

fn expand_suggestion_subdiagnostic(
    suggestion: &SuggestionKind,
    fields: &SubdiagFields,
) -> Result<TokenStream2> {
    match suggestion {
        SuggestionKind::Single(suggestion) => {
            let primary = fields.primary_node.as_ref().ok_or_else(|| {
                Error::new(
                    ProcSpan::call_site(),
                    "#[suggestion(...)] subdiagnostics require a #[primary_node] field",
                )
            })?;

            ensure_no_suggestion_parts(fields)?;
            let message = format_template(&suggestion.message)?;
            let code = format_template(&suggestion.code)?;

            expand_subdiagnostic_node_apply(primary, quote! {
                let snippet = ::annotate_snippets::Snippet::source(&src.contents)
                    .path(src.path.display().to_string())
                    .patch(::annotate_snippets::Patch::new(node.range.into(), #code));
                groups.push(
                    ::annotate_snippets::Group::with_title(
                        ::annotate_snippets::Level::HELP.secondary_title(#message)
                    )
                    .element(snippet)
                );
            })
        }
        SuggestionKind::Multipart(suggestion) => {
            let message = format_template(&suggestion.message)?;
            if fields.suggestion_parts.is_empty() {
                return Err(Error::new(
                    ProcSpan::call_site(),
                    "#[multipart_suggestion(...)] requires at least one #[suggestion_part(...)] field",
                ));
            }

            let patches = fields.suggestion_parts.iter().map(|part| {
                let binding = &part.binding;
                let code = format_template(&part.code)?;
                if is_type_named(&part.ty, "Node") {
                    Ok(quote! {
                        (
                            #binding,
                            ::annotate_snippets::Patch::new(#binding.range.into(), #code),
                        )
                    })
                } else {
                    Err(Error::new(
                        part.ty.span(),
                        "#[suggestion_part(...)] is only supported on Node fields",
                    ))
                }
            }).collect::<Result<Vec<_>>>()?;

            Ok(quote! {
                {
                    let snippets = crate::diag::patch_snippets(ctx, [ #( #patches ),* ]);
                    groups.push(
                        ::annotate_snippets::Group::with_title(
                            ::annotate_snippets::Level::HELP.secondary_title(#message)
                        )
                        .elements(snippets)
                    );
                }
            })
        }
    }
}

fn ensure_no_suggestion_parts(fields: &SubdiagFields) -> Result<()> {
    if fields.suggestion_parts.is_empty() {
        Ok(())
    } else {
        Err(Error::new(
            ProcSpan::call_site(),
            "#[suggestion(...)] cannot be combined with #[suggestion_part(...)] fields",
        ))
    }
}

fn primary_annotation_tokens(range: TokenStream2, labels: &[LitStr]) -> TokenStream2 {
    let mut iter = labels.iter().map(format_template);

    match iter.next() {
        Some(Ok(first)) => {
            let extra = iter.map(|label| {
                let label = label.unwrap();
                quote! {
                    .annotation(
                        ::annotate_snippets::AnnotationKind::Context
                            .span(#range)
                            .label(#label)
                    )
                }
            });

            quote! {
                .annotation(
                    ::annotate_snippets::AnnotationKind::Primary
                        .span(#range)
                        .label(#first)
                )
                #( #extra )*
            }
        }
        Some(Err(err)) => err.to_compile_error(),
        None => quote! {
            .annotation(::annotate_snippets::AnnotationKind::Primary.span(#range))
        },
    }
}

fn context_annotation_tokens(range: TokenStream2, labels: &[LitStr]) -> TokenStream2 {
    let annotations = labels.iter().map(|label| {
        let label = format_template(label).unwrap();
        quote! {
            .annotation(
                ::annotate_snippets::AnnotationKind::Context
                    .span(#range)
                    .label(#label)
            )
        }
    });

    quote! { #( #annotations )* }
}

fn parse_lit_attr(attr: &Attribute, name: &str) -> Result<LitStr> {
    let lit = attr
        .parse_args::<LitStr>()
        .map_err(|_| Error::new(attr.span(), format!("expected #[{name}(\"...\")]")))?;
    let _ = parse_template_exprs(&lit)?;
    Ok(lit)
}

fn parse_field_attrs(field: &Field) -> Result<FieldAttrs> {
    let mut attrs = FieldAttrs::default();

    for attr in &field.attrs {
        if attr.path().is_ident("primary_node") {
            if attrs.primary_node {
                return Err(Error::new(attr.span(), "duplicate #[primary_node]"));
            }
            attrs.primary_node = true;
        } else if attr.path().is_ident("label") {
            attrs.labels.push(parse_lit_attr(attr, "label")?);
        } else if attr.path().is_ident("note") {
            attrs.notes.push(FieldMessage {
                level: MessageLevel::Note,
                message: parse_lit_attr(attr, "note")?,
            });
        } else if attr.path().is_ident("help") {
            attrs.notes.push(FieldMessage {
                level: MessageLevel::Help,
                message: parse_lit_attr(attr, "help")?,
            });
        } else if attr.path().is_ident("suggestion") {
            attrs.suggestion = true;
        } else if attr.path().is_ident("suggestion_part") {
            if attrs.suggestion_part.is_some() {
                return Err(Error::new(attr.span(), "duplicate #[suggestion_part(...)]"));
            }
            attrs.suggestion_part = Some(attr.parse_args::<SuggestionPartAttr>()?);
        } else if attr.path().is_ident("subdiagnostic") {
            if attrs.subdiagnostic {
                return Err(Error::new(attr.span(), "duplicate #[subdiagnostic]"));
            }
            attrs.subdiagnostic = true;
        } else if attr.path().is_ident("diag") {
            return Err(Error::new(
                attr.span(),
                "field-level #[diag(...)] is not supported",
            ));
        }
    }

    Ok(attrs)
}

#[derive(Default)]
struct FieldAttrs {
    primary_node: bool,
    suggestion: bool,
    suggestion_part: Option<SuggestionPartAttr>,
    subdiagnostic: bool,
    labels: Vec<LitStr>,
    notes: Vec<FieldMessage>,
}

struct FieldMessage {
    level: MessageLevel,
    message: LitStr,
}

enum MessageLevel {
    Note,
    Help,
}

enum SubdiagnosticKind {
    Label(LitStr),
    Note(LitStr),
    Help(LitStr),
    Suggestion(SuggestionKind),
}

struct SubdiagFields {
    primary_node: Option<SubdiagPrimaryNode>,
    suggestion_parts: Vec<SuggestionPartField>,
}

struct SubdiagPrimaryNode {
    binding: Ident,
    ty: Type,
}

struct SuggestionPartField {
    binding: Ident,
    ty: Type,
    code: LitStr,
}

enum SuggestionKind {
    Single(SuggestionAttr),
    Multipart(MultipartSuggestionAttr),
}

impl MessageLevel {
    fn tokens(&self) -> TokenStream2 {
        match self {
            Self::Note => quote!(NOTE),
            Self::Help => quote!(HELP),
        }
    }
}

fn format_template(lit: &LitStr) -> Result<TokenStream2> {
    let parsed = parse_template_exprs(lit)?;
    let fmt = LitStr::new(&parsed.format_string, lit.span());
    let args = parsed.args;

    Ok(if args.is_empty() {
        quote! { format!(#fmt) }
    } else {
        quote! { format!(#fmt, #( #args ),*) }
    })
}

fn parse_template_exprs(lit: &LitStr) -> Result<ParsedTemplate> {
    let s = lit.value();
    let mut format_string = String::with_capacity(s.len());
    let mut args = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        match ch {
            '{' => {
                if matches!(chars.peek(), Some((_, '{'))) {
                    let _ = chars.next();
                    format_string.push_str("{{");
                    continue;
                }

                if !matches!(chars.peek(), Some((_, '$'))) {
                    format_string.push('{');
                    continue;
                }

                let _ = chars.next();

                let expr_start = idx + ch.len_utf8() + '$'.len_utf8();
                let mut expr_end = expr_start;
                let mut brace_depth = 0usize;
                let mut paren_depth = 0usize;
                let mut bracket_depth = 0usize;
                let mut in_string = false;
                let mut in_char = false;
                let mut escaped = false;
                let mut found_end = false;

                for (inner_idx, inner_ch) in chars.by_ref() {
                    if in_string {
                        if escaped {
                            escaped = false;
                        } else if inner_ch == '\\' {
                            escaped = true;
                        } else if inner_ch == '"' {
                            in_string = false;
                        }
                        continue;
                    }

                    if in_char {
                        if escaped {
                            escaped = false;
                        } else if inner_ch == '\\' {
                            escaped = true;
                        } else if inner_ch == '\'' {
                            in_char = false;
                        }
                        continue;
                    }

                    match inner_ch {
                        '"' => in_string = true,
                        '\'' => in_char = true,
                        '{' => brace_depth += 1,
                        '}' => {
                            if brace_depth > 0 {
                                brace_depth -= 1;
                            } else if paren_depth == 0 && bracket_depth == 0 {
                                expr_end = inner_idx;
                                found_end = true;
                                break;
                            }
                        }
                        '(' => paren_depth += 1,
                        ')' => paren_depth = paren_depth.saturating_sub(1),
                        '[' => bracket_depth += 1,
                        ']' => bracket_depth = bracket_depth.saturating_sub(1),
                        _ => {}
                    }
                }

                if !found_end {
                    return Err(Error::new(lit.span(), "unclosed `{$...}` in diagnostic template"));
                }

                let expr_src = s[expr_start..expr_end].trim().to_string();

                if expr_src.is_empty() {
                    return Err(Error::new(lit.span(), "empty `{$}` in diagnostic template"));
                }

                let expr = syn::parse_str::<Expr>(&expr_src).map_err(|err| {
                    Error::new(
                        lit.span(),
                        format!("invalid expression `{{${expr_src}}}` in diagnostic template: {err}"),
                    )
                })?;

                format_string.push_str("{}");
                args.push(expr);
            }
            '}' => format_string.push('}'),
            _ => format_string.push(ch),
        }
    }

    Ok(ParsedTemplate { format_string, args })
}

struct ParsedTemplate {
    format_string: String,
    args: Vec<Expr>,
}

fn is_type_named(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(tp) => tp.path.segments.last().is_some_and(|seg| seg.ident == name),
        _ => false,
    }
}

fn is_option_of_named(ty: &Type, inner_name: &str) -> bool {
    inner_type_of(ty, "Option").is_some_and(|inner| is_type_named(inner, inner_name))
}

fn is_vec_of_named(ty: &Type, inner_name: &str) -> bool {
    inner_type_of(ty, "Vec").is_some_and(|inner| is_type_named(inner, inner_name))
}

fn is_option_type(ty: &Type) -> bool {
    is_type_named(ty, "Option")
}

fn is_vec_type(ty: &Type) -> bool {
    is_type_named(ty, "Vec")
}

fn inner_type_of<'a>(ty: &'a Type, container: &str) -> Option<&'a Type> {
    let Type::Path(tp) = ty else {
        return None;
    };

    let seg = tp.path.segments.last()?;
    if seg.ident != container {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };

    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

struct SuggestionAttr {
    message: LitStr,
    code: LitStr,
}

impl Parse for SuggestionAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let message: LitStr = input.parse()?;

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
                "code" => code = Some(value),
                "applicability" => {}
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unknown suggestion key `{other}`"),
                    ));
                }
            }
        }

        Ok(Self {
            message: message.clone(),
            code: code.ok_or_else(|| {
                Error::new(
                    message.span(),
                    "missing `code = \"...\"` in #[suggestion(...)]",
                )
            })?,
        })
    }
}

struct MultipartSuggestionAttr {
    message: LitStr,
}

impl Parse for MultipartSuggestionAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let message: LitStr = input.parse()?;

        while !input.is_empty() {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let _: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "applicability" => {}
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unknown multipart suggestion key `{other}`"),
                    ));
                }
            }
        }

        Ok(Self { message })
    }
}

struct SuggestionPartAttr {
    code: LitStr,
}

impl Parse for SuggestionPartAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let key: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value: LitStr = input.parse()?;

        if key != "code" {
            return Err(Error::new(
                key.span(),
                "expected #[suggestion_part(code = \"...\")]",
            ));
        }

        if !input.is_empty() {
            let _: Token![,] = input.parse()?;
            if !input.is_empty() {
                return Err(Error::new(
                    input.span(),
                    "unexpected tokens in #[suggestion_part(...)]",
                ));
            }
        }

        Ok(Self { code: value })
    }
}
