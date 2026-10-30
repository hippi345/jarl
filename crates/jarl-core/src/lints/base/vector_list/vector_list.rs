use crate::diagnostic::*;
use crate::rule_set::Rule;
use crate::utils::{
    Formals, StatementsExt, get_arg, get_arg_by_name, get_function_name, node_contains_comments,
};
use air_r_syntax::*;
use biome_rowan::{AstNode, AstSeparatedList};
use oak_core::syntax_ext::RIdentifierExt;

const FORMALS_LAPPLY: Formals = &["X", "FUN"];
const FORMALS_SAPPLY: Formals = &["X", "FUN"];

/// <!-- docs: start -->
/// Version added: 0.8.0
///
/// ## What it does
///
/// Checks for usage of `lapply()` or `sapply()` with a function that always
/// returns `NULL`, which is often used to pre-allocate a list of `NULL` values.
///
/// ## Why is this bad?
///
/// Calling `lapply()` or `sapply()` runs R code for every element of the input,
/// which is much slower and more memory-hungry than creating the list directly
/// with `vector("list", ...)`.
///
/// The automated fix is marked unsafe because replacing `sapply()` can change
/// the names on the result when `USE.NAMES = TRUE`.
///
/// ## Example
///
/// ```r
/// lapply(seq_len(n), function(x) NULL)
/// sapply(n, function(x) NULL, USE.NAMES = FALSE)
/// ```
///
/// Use instead:
/// ```r
/// vector("list", n)
/// vector("list", length(n))
/// ```
///
/// ## References
///
/// See `?vector`
/// <!-- docs: end -->
pub fn vector_list(
    ast: &RCall,
    fn_name: &str,
    ns_prefix: Option<&str>,
) -> anyhow::Result<Option<Diagnostic>> {
    if !matches!(fn_name, "lapply" | "sapply") || ns_prefix.is_some_and(|ns| ns != "base::") {
        return Ok(None);
    }

    let args = ast.arguments()?.items();
    let formals = if fn_name == "lapply" {
        FORMALS_LAPPLY
    } else {
        FORMALS_SAPPLY
    };

    let arg_x = get_arg(ast, formals, "X");
    let arg_fun = get_arg(ast, formals, "FUN");

    let fun_value = unwrap_or_return_none!(arg_fun.and_then(|arg| arg.value()));
    if !fun_returns_null(fun_value) {
        return Ok(None);
    }

    if fn_name == "lapply" {
        if args.iter().count() != 2 {
            return Ok(None);
        }
    } else if !sapply_is_list_like(&args)? {
        return Ok(None);
    }

    let x_value = unwrap_or_return_none!(arg_x.and_then(|arg| arg.value()));
    let length_expr = unwrap_or_return_none!(list_length_expr(&x_value));

    let range = ast.syntax().text_trimmed_range();
    let fix = format!(r#"vector("list", {length_expr})"#);
    let fn_label = if fn_name == "lapply" {
        "`lapply()`"
    } else {
        "`sapply()`"
    };

    let diagnostic = Diagnostic::new(
        ViolationData::new(
            Rule::VectorList,
            format!("{fn_label} with a function that returns `NULL` is inefficient."),
            Some(r#"Use `vector("list", ...)` instead."#.to_string()),
        ),
        range,
        Fix::new(range, fix, node_contains_comments(ast.syntax())),
    );

    Ok(Some(diagnostic))
}

/// Whether `sapply()` is used like `lapply()` to build a list of `NULL` values.
fn sapply_is_list_like(args: &RArgumentList) -> anyhow::Result<bool> {
    let simplify = get_arg_by_name(args, "simplify");
    let use_names = get_arg_by_name(args, "USE.NAMES");

    for arg in args.iter().flatten() {
        let Some(name_clause) = arg.name_clause() else {
            continue;
        };
        let Ok(name) = name_clause.name() else {
            return Ok(false);
        };
        let allowed = matches!(
            name,
            AnyRArgumentName::RIdentifier(ident) if ident.name_text() == "simplify"
                || ident.name_text() == "USE.NAMES"
        );
        if !allowed {
            return Ok(false);
        }
    }

    if let Some(simplify) = simplify
        && let Some(value) = simplify.value()
        && value.to_trimmed_string() != "FALSE"
    {
        return Ok(false);
    }

    if let Some(use_names) = use_names
        && let Some(value) = use_names.value()
        && value.to_trimmed_string() == "TRUE"
    {
        return Ok(false);
    }

    Ok(true)
}

fn fun_returns_null(fun: AnyRExpression) -> bool {
    if fun.as_r_null_expression().is_some() {
        return true;
    }

    let Some(defn) = fun.as_r_function_definition() else {
        return false;
    };

    let Ok(body) = defn.body() else {
        return false;
    };

    match body {
        AnyRExpression::RNullExpression(_) => true,
        AnyRExpression::RBracedStatements(braced) => match braced.expressions().as_slice() {
            [] => true,
            [expr] => expr.as_r_null_expression().is_some(),
            _ => false,
        },
        _ => false,
    }
}

fn list_length_expr(x: &AnyRExpression) -> Option<String> {
    if let Some(call) = x.as_r_call() {
        let function = call.function().ok()?;
        if get_function_name(function) == "seq_len" {
            let items = call.arguments().ok()?.items();
            if items.iter().count() == 1 {
                let arg = items.into_iter().next()?.ok()?;
                if arg.name_clause().is_none() {
                    return arg.value().map(|value| value.to_trimmed_string());
                }
            }
        }
    }

    Some(format!("length({})", x.to_trimmed_string()))
}
