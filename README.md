# porth-rs

This is a Rust implementation of some of the most basic features of the [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) programming language [Porth](https://gitlab.com/tsoding/porth) created by Tsoding/Rexim. The documented development of the original Porth can be found [here](https://www.youtube.com/playlist?list=PLpM-Dvs8t0VbMZA7wW9aR3EtBqe2kinu4).

## Example Usage

The file provided will be compiled into x86-64 fasm which will be written to `./out.asm` and can then be compiled to an executable binary with [fasm](https://flatassembler.net/)

```console
$ cargo run -- main.porth
$ fasm out.asm
$ chmod +x out
$ ./out
9
```

## Features

Only a very limited number of very basic features are implemented, namely:

- PushInt
- Plus
- Minus
- Print

See `./main.porth` for an example program.

## Disclaimer

The original author has expressed frustrations with competing compilers. I would like to say that I am a big fan of the Tsoding project, and the work in this repository is merely a reflection of that and does not have any aims of competition. It was created for the joy of programming while I am learning about ideas in compiler development.

The code in this repository was written a couple of months before publishing, however I decided to publish it in the end as the original Porth code is licensed under MIT and, like I said, there is no aim to compete.

If I continue to work on this then it will very likely diverge from the original project.
