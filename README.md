# Rack

This is a Rust implementation of my toy [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) programming language Rack. It is heavily inspired by the language [Porth](https://gitlab.com/tsoding/porth) created by Tsoding/Rexim. The documented development of the original Porth can be found [here](https://www.youtube.com/playlist?list=PLpM-Dvs8t0VbMZA7wW9aR3EtBqe2kinu4).

## Usage

```console
USAGE:
    rackc [OPTIONS] <FILE>

ARGS:
    <FILE>    Input file

OPTIONS:
    -h, --help               Print help information
    -o, --out <FILE>         Output file
    -q, --quiet              Don't print log information
    -r, --run                Run the program after successful compilation
    -t, --target <TARGET>    Target architecture [default: x86_64-linux]
```

### Targets
- `x86_64-linux`\*
- `x86_64-fasm`
- [`mos_6502-nesulator`](https://github.com/Piturnah/nesulator)

\* Requires [fasm](https://flatassembler.net/download.php) on path (on most package managers)

## Example Usage

The file provided will be compiled into x86-64 fasm which will be written to `./out.asm` and can then be compiled to an executable binary with [fasm](https://flatassembler.net/)

```console
$ cargo run -- examples/hello.rk -r
hello, world!
```

## Documentation

### Functions

Rack code goes inside a function, declared as follows:

```
fn foo in
  // do some stuff
end
```

You can call it by using the function's name:

```
fn main in
  foo
end
```

Early return is achieved with `ret` keyword.

#### PushInt

Push a u64 onto the stack.

```
42       print
0x2a     print
0o52     print
0b101010 print
'*'      print
```

```console
42
42
42
42
42
```

#### String Literals

String literals are pushed to the stack as a count followed by a pointer.

```
"Hello, world!" print print
```

```console
4198733
13
```

### Output

#### print

Pops from the stack and sends to stdout as u64.

```
42 print
```

```console
42
```

#### puts

Pops a pointer and count from the stack and prints the string at the pointer to stdout.

```
"Hello, world!\n" puts
```

```console
Hello, world!
```

### Stack Manipulation

#### let

Bind arbitrary number of stack elements to names

```
1 2 3
let a b c in
  a print
  b print
  c print
end
```

```console
1
2
3
```

#### peek

Like `let`, but the elements remain on the stack.

```
1 2 3
peek a b c in end
print print print
```

```console
3
2
1
```

#### drop

Drops one element from the stack

```
42 84 drop print
```

```console
42
```

#### swap

Swaps the two elements at the top of the stack


```
42 84 swap print print
```

```console
42
84
```

#### dup

Duplicates the element at the top of the stack

```
42 dup print print
```

```console
42
42
```

### Arithmetic

#### +

Pops two elements from the stack, pushes the result of adding them

```
20 21 + print
```

```console
42
```

#### -

Pops `a` then `b` from the stack, pushes the result of `b - a`

```
60 18 - print
```

```console
42
```

#### /

Pops `a` then `b` from the stack, pushes the result of `b / a`

```
10 2 / print
```

```console
5
```

#### %

Pops `a` then `b` from the stack, pushes the result of `b % a`

```
10 4 % print
```

```console
2
```

#### divmod

Pops `a` then `b`, pushes the result of `b / a` then `b % a`

```
10 3 divmod 
let quotient remainder in
  quotient print
  remainder print
end
```

```console
3
1
```

### Control Flow

#### if \<branch> end

Pops from stack. If `true`, execute `<branch>`, else go to `end`.

```
true if
  42 print
end

false if
  84 print
end
```

```console
42
```

#### while \<condition> do \<branch> end

While `<condition>` is `true`, execute `branch`.

```
0 while 1 + dup 6 < do
  dup print
end
drop
```

```console
1
2
3
4
5
```

### Logic

#### true

Pushes `1` onto the stack.

#### false

Pushes `0` onto the stack.

#### =

Pops `a` and `b` from stack, pushes boolean result of `a == b`

```
3 3 = print
```

```console
1
```

#### !=

Pops `a` and `b` from stack, pushes boolean result of `a != b`

```
3 3 != print
```

```console
0
```

#### not

Inverts boolean value on the stack

#### and

Pops `a` and `b`, pushes `1` if both `a` and `b` are `1`, `0` otherwise.n

#### or

Pops `a` and `b`, pushes `1` if one of `a` and `b` is `1`, `0` otherwise.

#### <

Pops `a` then `b`, pushes `1` if `b < a`, `0` otherwise.

```
1 2 < if
  42 print
end
```

```console
42
```

#### >

Pops `a` then `b`, pushes `1` if `b > a`, `0` otherwise. See `<`.

### Memory

#### @

Reads a single byte from memory at the address stored on the stack.

```
"hello" @ print
```

```console
104
```

### Comments

Comments are denoted by `//`. Must be separated by space. e.g `1 // comment`
