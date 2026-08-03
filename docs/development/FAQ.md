# FAQ

**Why Lefthook instead of Husky?**  
Polyglot monorepo; hooks must not require Node for Rust/Go-only commits.

**Why `just` instead of only Make?**  
Clearer recipes for a multi-language repo; Make remains as thin wrappers.

**Why no business seed SQL?**  
No business schema yet — fixtures document demo data safely.

**Where is Swagger?**  
`/docs` on the API; Redoc at `/redoc`.

**What is the required CI check?**  
`PR Validation`.
