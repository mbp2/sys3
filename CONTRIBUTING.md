# Contributing to the Trident Project

Contributing to the Trident 3 project can take many forms:
- You may contribute code to the core project, the bootloader, or any other MBP2-related project;
- You can offer localisation services in the form of language translations, interface design, or other facets;
- You can join the [PINE Community](https://discord.gg/ZGx5kbtEwQ) to interact with the core team;
- You may contribute to overall documentation;
- You may also proffer implementation and design details for future feature discussions.

### Contributing Code

We have a small set of aesthetic rules in our code:
- Function/method and variable names should generally be `snake_case`, although `PascalCase` is acceptable for public methods;
- Type names should always be `PascalCase`;
- Every module should be indented with *three (3) spaces*, and no more or less;
- Curly braces should always be on the same line as closing parenthesis unless you have applied type constraints;
- Generic type parameters should be more descriptive than `T`, though `T` is acceptable for types where the generic is self-explanatory;
- Struct fields should generally be `pub` unless there is a good reason to keep them private;
- Keep it declarative and functional; imperative when necessary.

### I Don't Know What I'm Talking About

So we have established by this point that I don't know what I am talking about, so I would greatly appreciate your help
in fixing that particular issue! Please and thank you!

- Az
