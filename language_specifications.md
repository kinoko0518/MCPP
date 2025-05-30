# Language Specification
## Syntaxs
### Variable Definement
The keyword of definement is ```let```. It has bacic auto type guessing feature, but if you want or it won't be determined to a type, you can or have to add type specialiser by ```:[type name]```. Here's a example:
```
// It will be typed to int
let foo = 12;
// It will be typed to float
let bar = foo * 0.3;
// Define variable with typing
let baz:float = 5; // 5.0
```

### Function Definement
The keyword of function definement is ```fn```. It expects ```(``` token, 0 or more arguments with typing and ```)``` token. Here's an example.
```
fn hello_world_for_n_times(n:int) {
    let counter = 0;
    while counter <= n {
        counter = counter + 1;
        native!("say Hello world!");
    }
}
```

### Comment Out
You can comment out a line by ```//``` to the end of line.
```
// It works
let qux = 8;
// It won't work
// let quux = 9;
// It will be 17 because of 15 and semicolon
// will be ignored
let quuux = // 15;
17;
```

### Code Block
You can make a code block by sorround ```{}```. The variables defined in a code block will be freed at the end of a code block, and a code block is actually corresponding to a ```mcfunction``` file.

### Types and Operations
- ```int``` : It is corresponding to a real value of a scoreboard.
- ```float```: It is corresponding to 1000 times multipled value of a scoreboard. The calcations that occured between int type and float type will be automatically scaled at the time of compiling.
- ```bool```: It is corresponding to 0 or not. **Be attention to true isn't corresponding to 1!**
#### Arithmetic Operations
- ```+, -, *, /``` between numeric types are fully supported.
- You can only apply ```%``` to only ```int and int``` type.
#### Comparison Operations
- ```==, !=``` between same types are fully supported.
- ```<, <=, ==, !=, >=, >``` between numeric types are fully supported.
#### Logical Operations
- ```&, |``` between booleans are fully supported.
- ```!``` is planned but unimplicated.

### Control Syntax
#### If Syntax
The keyword of a if syntax is ```if``` and it expects a formula and a code block. ```else``` and ```else if``` is planned, but unimplicated. Here's an example.
```
let quuuux = 15;
// It won't be called on runtime!
if quuuux == 5 {
    native!("say hello world");
}
```
#### While Syntax
The keyword of a if syntax is ```while``` and it expects a formula and a code block. Here's an example.
```
let quuuuux = 0;
while quuuuux <= 16 {
    quuuuux = quuuuux + 1;
    native!("say hello world");
}
```

### Macros
#### native!(```type```, ```command:str```)
```native!``` macro is a macro for use native command of the Minecraft. You can specify the returning type with ```type``` argument. If it doesn't have returning value or isn't neccessary, ```none``` to ignore returning value.