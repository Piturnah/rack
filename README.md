# porth-rs

This is a Rust implementation of some of the most basic features of the [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) programming language [Porth](https://gitlab.com/tsoding/porth) created by Tsoding/Rexim. The documented development of the original Porth can be found [here](https://www.youtube.com/playlist?list=PLpM-Dvs8t0VbMZA7wW9aR3EtBqe2kinu4).

## Usage

```console
USAGE:
    compiler [OPTIONS] <FILE>

ARGS:
    <FILE>    Input file

OPTIONS:
    -h, --help    Print help information
    -r, --run     Run the program after successful compilation
```

## Example Usage

The file provided will be compiled into x86-64 fasm which will be written to `./out.asm` and can then be compiled to an executable binary with [fasm](https://flatassembler.net/)

```console
$ cargo run -- -r tests/if.porth
42
11
```

## Documentation

### Data

#### PushInt

Push a u64 onto the stack

**Example:** Push the integer 42 onto the stack

```
42
```

**Example** Hex literals are parsed too

```
0xff
```

### Stack Manipulation

#### print

Pops from the stack and sends to stdout as u64.

**Example**

```
42 print
```

**Output**

```console
42
```

#### drop

Drops one element from the stack

**Example**

```
42 84 drop print
```

**Output**

```console
42
```

#### swap

Swaps the two elements at the top of the stack

**Example**

```
42 84 swap print print
```

**Output**

```console
42
84
```

#### dup

Duplicates the element at the top of the stack

**Example**

```
42 dup print print
```

**Output**

```console
42
42
```

### Arithmetic

#### +

Pops two elements from the stack, pushes the result of adding them

**Example**

```
20 21 + print
```

**Output**

```console
42
```

#### -

Pops `a` then `b` from the stack, pushes the result of `b - a`

**Example**

```
60 18 - print
```

**Output**

```console
42
```

### Control Flow

#### if \<branch> end

Pops from stack. If `true`, execute `<branch>`, else go to `end`.

**Example**

```
true if
  42 print
end

false if
  84 print
end
```

**Output**

```console
42
```

#### while \<condition> do \<branch> end

While `<condition>` is `true`, execute `branch`.

**Example**

```
0 while 1 + dup 6 < do
  dup print
end
drop
```

**Output**

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

**Example**

```
3 3 = print
```

**Output**

```console
1
```

#### !=

Pops `a` and `b` from stack, pushes boolean result of `a != b`

**Example**

```
3 3 != print
```

**Output**

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

**Example**

```
1 2 < if
  42 print
end
```

**Output**

```console
42
```

#### >

Pops `a` then `b`, pushes `1` if `b > a`, `0` otherwise. See `<`.

### Comments

Comments are denoted by `//`. Must be separated by space. e.g `1 // comment`

## Disclaimer

The original author has expressed frustrations with competing compilers. I would like to say that I am a big fan of the Tsoding project, and the work in this repository is merely a reflection of that and does not have any aims of competition. It was created for the joy of programming while I am learning about ideas in compiler development.

The code in this repository was written a couple of months before publishing, however I decided to publish it in the end as the original Porth code is licensed under MIT and, like I said, there is no aim to compete.

If I continue to work on this then it will very likely diverge from the original project.
