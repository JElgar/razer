# razer

## Start

```
docker-compose up -d
diesel migration run
pnpm i
pnpm dlx tailwindcss -i styles/tailwind.css -o assets/main.css --watch
cargo watch -x run
```

## Modules

### Admin

- Should be possible to use with existing orms and custom implementation of get, list, update.

## Form Builder

When registering admin ensure type and input type provided. Type must implement CreateFrom<InputType>. Input type must dervice input type and user must provide a built version of InputTypeFormBuilder

InputTypeFormBuilder will be generated for each input with dervice macro AdminInput. This will

- Create a form builder which uses PhantonData and generics to track that all required fields have been provided to the builder. Each form field setter will take in a form field for the required type (e.g. a string field/ a date field).

Ideally a default form can be generated if the user wants and then they can simply override bits as they wish (more like other models) 

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

```rust

admin_router
    .register<MyModel>()

```
