# razer

## Start

```
pnpm dlx tailwindcss -i styles/tailwind.css -o assets/main.css --watch
cargo watch -x run
```
## TODO
- [ ] Split stored type with input type (Similar to Insertable logic in diesel - so saved type does not have to match inserted type)
- [ ] Crate adapter for at least 1 ORM (maybe start with diesel)
- [ ] Authentication
- [ ] UI
- [ ] Would be nice to support other routers (other than axum)

## Notes

- https://joeymckenzie.tech/blog/templates-with-rust-axum-htmx-askama/
