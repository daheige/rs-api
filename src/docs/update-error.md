# axum 0.8后报错
```
error[E0432]: unresolved import `axum::async_trait`
 --> src/handlers/json_or_form.rs:2:5
  |
2 |     async_trait,
  |     ^^^^^^^^^^^ no `async_trait` in the root
  
    --> src/handlers/validate_form.rs:23:14
   |
23 |     async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ lifetimes do not match associated function in trait
```

去掉#[async_trait]即可，在axum 0.8后不再需要这个derive注解
