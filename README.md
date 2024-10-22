# TypePipe!

### Ever wanted the ability to curry type signatures?
Then wait no longer!
With this crate, you can do things like:

```rust
type_pipe! [
    T,
    MyType<_>,
    MyWrapper<_>,
]
```
and this produces:
```rust
MyWrapper<MyType<T>>,
```

There's three main macros:

`type_pipe`: This replaces all `_` with the resulting type from the previous line.  
This is the example I demonstrated above.
Another example:
* `T, Wrapped<_, String>, Outer<String, _>` -> `Outer<String, Wrapped<T, String>>`

`type_pipe_pre`: This inserts the resulting type from the previous line into the first position.  
 * `T, Wrapped, Outer<String>` -> `Outer<Wrapped<T>, String>`
 * `T, Wrapped<String>, Outer<String>` -> `Outer<Wrapped<T, String>, String>`

`type_pipe_post`: This inserts the resulting type from the previous line into the last position.
* `T, Wrapped, Outer<String>` -> `Outer<String, Wrapped<T>>`
