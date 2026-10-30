pub(crate) mod vector_list;

#[cfg(test)]
mod tests {
    use crate::utils_test::*;
    use insta::assert_snapshot;

    fn snapshot_lint(code: &str) -> String {
        format_diagnostics(code, "vector_list", None)
    }

    #[test]
    fn test_no_lint_vector_list() {
        expect_no_lint("vector(\"list\", n)", "vector_list", None);
        expect_no_lint("lapply(x, sum)", "vector_list", None);
        expect_no_lint("lapply(x, function(x) x)", "vector_list", None);
        expect_no_lint(
            "lapply(x, function(x) {\n  NULL\n  1\n})",
            "vector_list",
            None,
        );
        expect_no_lint(
            "lapply(x, function(x) NULL, extra = 1)",
            "vector_list",
            None,
        );
        expect_no_lint(
            "sapply(x, function(x) NULL, USE.NAMES = TRUE)",
            "vector_list",
            None,
        );
        expect_no_lint(
            "sapply(x, function(x) NULL, simplify = TRUE)",
            "vector_list",
            None,
        );
        expect_no_lint("sapply(x, sum)", "vector_list", None);
        expect_no_lint("purrr::map(x, function(x) NULL)", "vector_list", None);
    }

    #[test]
    fn test_lint_vector_list() {
        assert_snapshot!(snapshot_lint("sapply(seq_len(n), function(x) NULL)"));
        assert_snapshot!(snapshot_lint("lapply(seq_len(n), function(x) NULL)"));
        assert_snapshot!(snapshot_lint("lapply(n, function(x) NULL)"));
        assert_snapshot!(snapshot_lint(
            "sapply(seq_len(n), function(x) NULL, USE.NAMES = FALSE)"
        ));
        assert_snapshot!(snapshot_lint("lapply(x, function(x) { NULL })"));
    }

    #[test]
    fn test_vector_list_fix() {
        assert_snapshot!(get_unsafe_fixed_text(
            vec![
                "lapply(seq_len(n), function(x) NULL)",
                "lapply(n, function(x) NULL)",
                "sapply(n, function(x) NULL, USE.NAMES = FALSE)",
            ],
            "vector_list",
        ));
    }

    #[test]
    fn test_vector_list_with_comments_no_fix() {
        assert_snapshot!(get_unsafe_fixed_text(
            vec![
                "# leading comment\nlapply(x, function(x) NULL)",
                "lapply(x, function(x) NULL) # trailing comment",
            ],
            "vector_list",
        ));
    }
}
