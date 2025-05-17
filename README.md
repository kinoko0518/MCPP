# What's MC++?
<img src="https://github.com/user-attachments/assets/41ada7b0-6caf-4139-bfd4-80ae67b7b9d6" width=300>

MC++ is a high level programming language that can be compiled to minecraft commands with these features:
## Features
- Modern variable system
  - Static typing system (int, float, bool)
  - Variables with scope
- Modern calcation
  - A formula in a line (e.g (0.55 + 0.1) * 12)
  - Boolean algebra supporting
## Planned
- Control syntaxs (if, while)
- Function
  - Definement
  - Callment
  - Arguments
  - Returning values
MC++ is providing an evaluater that can only evaluate formulas, for example,
```
let a:int = 2;
let b = 10.8;
let c:bool = (0.4 + 5) * a == b;
```

# Syntax
| Operation | Syntax |
| ---- | ---- |
| Variable definement | ```let foo``` |
| Variable definement with typing | ```let foo:bar``` |
| Value assignment | ```foo = bar``` |
## Example
⚠️It doesn't work on the current version!
```
fn multiple_2(target:int) -> int {
  let foo:int = 2;
  return target * a;
}
fn main() {
  // MC++
  let bar:float = 10.8;
  let baz:bool = multiple_2(0.4 + 5) == b; // True
}
```
