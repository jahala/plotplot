# Brand Architecture (umbrella + product layers)

A brand may be one entity (a flat `.brand/` — the zero-config default) or a family: an umbrella brand whose products each carry their own layer.

- **Umbrella**: the shared truth — palette, typography, layout, surface, voice, the organization's identity. Lives at `.brand/*.md`.
- **Product layers**: `.brand/products/<product>/` — deltas only (the product's accent claim, mark, tagline, product terminology). An absent file inherits the umbrella entirely. Never copy umbrella values down.
- **Resolution**: every read resolves umbrella ⊕ product. Product context comes from `--product` (`--product -` says there is none), else `.petalsrc`'s `product:` field, else the single directory under `products/`, else flat. Progressive disclosure holds: at most two files per dimension.
- **Cross-product accents**: each product's accent (umbrella Product Accents table) marks THAT product. Literal use of another product's accent in your product's UI is a warning, by name. Off-palette colors near a product accent are suggested with the owner named.
- The book, tokens, and check all operate on the RESOLVED brand; the book names each value's owning layer.
