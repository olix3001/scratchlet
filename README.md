# Scratchlet language
Scratchlet is a programming language that compiles into [scratch](https://scratch.mit.edu/) projects.

I've already experimented with this idea in the past (see [PurrLang](https://github.com/olix3001/PurrLang)) which proved my idea viable.
Previous attempt at this ended up working, but neither Its syntax nor implementation were worth continuing in my opinion.
Now, with more knowledge about programming language architecture and probably with more patience to keep to one project at a time, I've decided to give It another go.

This readme describes both language architecture and Its syntax, so unless there will be a better documentation in the future, you can reference this document instead.

> [!IMPORTANT]
> This project is not related by any means to [Scratch Foundation](https://www.scratchfoundation.org/). It is a hobby project made from passion to langdev.

## Language crates
This language's compiler is written in Rust, so It is divided into multiple crates.
This is the division of project code into multiple libraries and binaries.

| Crate | Description |
| --- | --- |
| [catnip](./catnip/) | This is something similar to cargo. CLI for compiling your projects. |
| [pawgen](./pawgen/) | Code generation for scratch projects. Wrapper around manually interacting with .sb3 json files. This also provides serde structures for the projects. |
| [scratchc](./scratchc/) | Compiler itself. It consists of frontend, MIR and backend. It also exports a binary for directly interacting with the compiler. |

## Project file
Project file - `Scratchlet.toml` - defines how your project works.
This file defines extensions, sprites, sounds and code roots.

Its syntax is the following:
```toml
[project]
extensions = ["pen", "ev3"] # Scratch/Turbowarp extensions to include.

[sprites.<name>]
costumes = { ... }
costume = "cat1"
root = "cat.sl"
position = { x = 0, y = 0 }
volume = 100
size = 100
direction = 90
visible = true
rotation_style = "all around"
layer = 1
draggable = false
```

## Language syntax

### Blocks
As it is in scratch, basic building blocks of the language are blocks.
But there is not a limited set of blocks on the scratch platform (thanks to extensions),
so block definitions are a part of language itself.

Basic scratch blocks and extensions are provided in "scratch" builtin library,
while turbowarp specific extensions are available under "turbowarp" library.

The language also allows for definition of custom blocks. Here is a syntax for this:
```
// Definition of square root block.
// You can explore project json to work this out.
block sqrt(x: number) -> number as operator_mathop {
  inputs: ${ NUM: x },
  fields: ${ OPERATOR: "sqrt" },
}
```

### Procedures
Scratch allows users to define their own blocks, you can achieve similar effect using procedures in scratchlet.
Procedures will be generated as custom blocks, however, when they are really short
compiler might decide to inline them. Other programming languages do this too.

Simple procedure that takes a name and greets user might look like this:
```
// Shorthand syntax. You can ommit return type here.
proc greet(name: text) = "Greetings, " + name + "!"

// Full syntax.
proc greet(name: text) -> text {
  return "Greetings, " + name + "!"
}
```

### Imports
Many languages require you to import everything separately, but in reality many
names are often imported from a single module. That is exactly why in scratchlet
imports are trees.

For example if you want to import `scratch::math::sqrt` and many other things from `scratch::looks` you can use this syntax:
```
import scratch::{
  math::sqrt,
  looks::{say, say_for}
}
```

### Structures
Structures allow you to combine multiple fields of different types into one data structure.
You can define them using the following syntax:
```
struct Vec2 {
  x: number,
  y: number,
}
```

They also allow implementations to define functions on them.
Variables with structure types are then compiled as multiple variables in the project.
Compiler will try to limit the amount of variables by reusing those which are no longer needed in a single procedure.

### Enums
Enums are essentially tagged unions. Their size (amount of variables) after compilation
is their largest variant.

### Traits
Those work similar to rust traits, might need more work to explain those here.

### Modules
Every file is essentially a separate module.
You can also define modules explicitly using `mod` keyword.
```
mod hello {
  proc world() = "Hello world"
}
```
