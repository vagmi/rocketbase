# Rocketbase

Rocketbase is an opinionated, Prisma-like database access library for Rust.

```
:warn: This project is completely broken at this point and is usable only by the author. Do not use
this.
```

## About

This project is inspired by [Prisma](https://www.prisma.io/)'s developer experience, but with a desire to tailor it for the Rust ecosystem and address some of its limitations.

### Key Features from Prisma
* **Single Schema File**: Define your database schema in a single, easy-to-read file.
* **Expressive DSL**: A powerful Domain-Specific Language for data modeling.
* **GraphQL-like Queries**: An intuitive interface for fetching data.

### Our Philosophy
* **No Database Ownership**: Database connection details are managed by your application, not the schema file.
* **Standard Types**: We prefer standard types like `uuid` over custom ones (e.g., CUIDs) for better interoperability.
* **Rust Native**: Built for Rust developers.

The goal is to provide a data access library with a "leaky abstraction," giving developers the power of a high-level API with the ability to drop down to lower levels when needed.

## Ideal Workflow

### 1. Define Your Schema

Create a `.schema` file to define your data models and relationships.

```prisma
# blog.schema
model User {
  id        uuid      @id
  email     string    @unique
  name      string?
  posts     Post[]
  comments  Comment[]
}

model Post {
  id        uuid      @id
  title     string
  content   text
  published_at timestamp?
  author    User @rel(fields: [author_id], references: [id])
  author_id uuid
  comments  Comment[]
}

model Comment {
  id        uuid      @id
  content   text
  author    User      @rel(fields: [author_id], references: [id])
  author_id uuid
  post      Post      @rel(fields: [post_id], references: [id])
  post_id   uuid
}
```

### 2. Query Your Data in Rust

Use the `query!` macro to build type-safe queries.

```rust
let rocket_base = rocketbase::RocketBase::init(pg_pool, schema_path)?;

let query = rocketbase::query!(rocket_base, {
    query {
        # filter by id (notice the singular)
        user(id="uuid-goes-here") {
            id
            email
            posts(first: 10)  {
                edges {
                    cursor
                }
               nodes {
                id
                content
                published_at
                comments_count # notice the relation (should resolve to a number)
               }
               pageInfo {
                count
                hasNext
               }
            }
        }
    }
});

// The generated type has a `call` method that executes the query.
let data = query().await?;
for post in data.user.posts {
    // process the posts
}
```

The `query!` macro is a procedural macro that generates Rust types from your query at compile time. It uses the parsed schema to understand your data models. At runtime, it converts the query into an efficient PostgreSQL query using JSON operations and deserializes the result into the generated types.

