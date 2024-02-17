# razer

## Start

```
pnpm dlx tailwindcss -i styles/tailwind.css -o assets/main.css --watch
cargo watch -x run
```

## Modules

### Admin

- Should be possible to use with existing orms and custom implementation of get, list, update.

## TODO
- [ ] Split stored type with input type (Similar to Insertable logic in diesel - so saved type does not have to match inserted type)
- [ ] Crate adapter for at least 1 ORM (maybe start with diesel)
- [ ] Authentication
- [ ] UI
- [ ] Would be nice to support other routers (other than axum)

## TODO
- Find out how to do partials/not have to provide all fields at the top level template
- Create integration with diesel
- Long Term: Create integrations with other orms/examples with sqlx
- Long Term: Create integrations with other http frameworks

## Notes

- https://joeymckenzie.tech/blog/templates-with-rust-axum-htmx-askama/
- https://github.com/mitsuhiko/minijinja
- https://github.com/silkenweb/silkenweb/blob/main/examples/htmx-axum/index.html
