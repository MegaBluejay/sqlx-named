# sqlx-named

Wrappers around [sqlx][] macros that allow using named parameters with them, even with database that don't support them (e.g. PostgreSQL).

## Examples

```rust,ignore
sqlx_named::query!(
  r#"select $answer "answer!""#,
  answer = 42,
)
```

Arguments without a name are treated as puns, with an extracted identifier getting used as a parameter name

```rust,ignore
let answer = 42;
let answer_ref = answer;
let anser_deref = &answer;

sqlx_named::query!(
  r#"select $answer "anwer!", $answer_ref "answer_ref!", $answer_deref "answer_deref!""#,
  answer as i32,
  &answer_ref,
  *answer_deref,
)
```

Also supported is a splat syntax for extracting multiple arguments from a single value

```rust,ignore
struct Args {
  fld: i32,
}

impl Args {
  fn meth(&self) -> i32 {
    self.fld + 1
  }
}

let args = Args { fld: 1 };

sqlx_named::query!(
  r#"select $fld "fld!", $meth "meth!""#,
  ..args {
    .fld,
    meth = .meth(),
  },
)
```

All `sqlx` macro variants are supported

```rust,ignore
sqlx_named::query_file_scalar_unchecked!(
  "./sql/some-query.sql",
  a = 1,
)
```

## Compatibility

This crate does not depend on [sqlx][], but major changes in the original macros' api could cause it to break.

Tested with version 0.8.2

[sqlx]: https://github.com/launchbadge/sqlx
