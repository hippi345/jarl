# vector_list

::: {.callout-note title="Added in 0.8.0" .low-opacity}
:::

## What it does

Checks for usage of `lapply()` or `sapply()` with a function that always
returns `NULL`, which is often used to pre-allocate a list of `NULL` values.

## Why is this bad?

Calling `lapply()` or `sapply()` runs R code for every element of the input,
which is much slower and more memory-hungry than creating the list directly
with `vector("list", ...)`.

The automated fix is marked unsafe because replacing `sapply()` can change
the names on the result when `USE.NAMES = TRUE`.

## Example

```r
lapply(seq_len(n), function(x) NULL)
sapply(n, function(x) NULL, USE.NAMES = FALSE)
```

Use instead:
```r
vector("list", n)
vector("list", length(n))
```

## References

See `?vector`
