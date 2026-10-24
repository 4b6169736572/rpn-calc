# rpn-calc

A terminal calculator that uses Reverse Polish Notation (RPN), written in Rust with [ratatui](https://ratatui.rs). Operators build expressions you can read on the stack, and `=` evaluates them.

## AI usage disclosure
Claude was used for generating documentation and in the initial planning of this project. All actual code written and architecture decisions were made by me.

## What is an RPN calculator?

Reverse Polish Notation puts the operator after its numbers: instead of `3 + 4` you type `3 4 +`. An RPN calculator keeps numbers on a stack. Each new number goes on top; an operator takes the top two, combines them, and puts the result back.

The order you enter things decides the order of operations, so RPN needs no parentheses and no precedence rules. Here is `(3 + 4) × 2`:

| You enter | Stack afterwards (top on the right) |
| --- | --- |
| `3` | 3 |
| `4` | 3 4 |
| `+` | 7 |
| `2` | 7 2 |
| `×` | 14 |

rpn-calc follows this model with one twist: an operator first builds an expression such as `3 + 4` on the stack, and `=` turns it into a number (see [Usage](#usage)).

## Getting started

You need Rust and Cargo; [rustup](https://rustup.rs) installs both. The crate uses the 2024 edition, so use a recent stable toolchain.

```sh
git clone https://github.com/4b6169736572/rpn-calc.git
cd rpn-calc
cargo run --release
```

To install the `rpn-calc` binary into `~/.cargo/bin` instead:

```sh
cargo install --git https://github.com/4b6169736572/rpn-calc
```

## Usage

Type a number and press `Enter` to push it. Operators join the top two items into an expression, and `=` works it out.

```text
   ┌ Stack ────────────────────────────────┐
 6 │                                       │
 5 │                                       │
 4 │                                       │
 3 │  (x * 2) + 1                          │
 2 │  (3 + 4) * 2                          │
 1 │  1                                    │
   └───────────────────────────────────────┘
┌ Input ───────────────────────────────────┐
>  10                                      │
└──────────────────────────────────────────┘
```

The newest item sits at the bottom of the Stack panel, at level 1. The numbers on the left are stack levels, and the Input line shows what you are typing.

### Keys

| Key | What it does |
| --- | --- |
| `0`–`9`, `.` | Type a number |
| `Enter` | Push the typed number; with nothing typed, duplicate the top item |
| `+` `-` `*` `/` `^` | Join the top two items into an expression (a typed number is pushed first) |
| `=` | Evaluate the top item: an expression becomes its value, a variable its stored value |
| `Backspace` | Drop the top item |
| `Tab` | Swap the top two items |
| `s` + letter | Store the typed number, or a copy of the top item, in a variable |
| `r` + letter | Recall a variable's value onto the stack |
| `'` + letter | Push the variable itself, to use in an expression |
| `Ctrl+N` + digit | Set a repeat count for the next `Enter` or `Tab` |
| `Esc` | Cancel a repeat count |
| `q`, `Ctrl+C` | Quit |

Variable names are single letters, `a`–`z` and `A`–`Z` except `e` and `E`. A variable can hold a number or a whole expression.

### Expressions

Operators don't calculate right away. The stack shows the expression in ordinary notation, with parentheses around nested parts, until you press `=`:

| Keys | Top of the stack |
| --- | --- |
| `3` `Enter` | `3` |
| `4` `+` | `3 + 4` |
| `2` `*` | `(3 + 4) * 2` |
| `=` | `14` |

To keep an expression after evaluating it, press `Enter` to duplicate it before `=`.

### Variables

Push a variable with `'`, build an expression around it, then give the variable a value and evaluate. While `s` waits for its letter, the Input line shows `s-`.

| Keys | Stack, newest last |
| --- | --- |
| `'` `x` `2` `*` `1` `+` | `(x * 2) + 1` |
| `5` `s` `x` | `(x * 2) + 1`, and x is now 5 |
| `Enter` `=` | `(x * 2) + 1`, `11` |
| `Tab` `7` `s` `x` `=` | `11`, `15` |

### Repeat counts

`Ctrl+N` and a single digit set a count for the next `Enter` or `Tab`. The Input line shows it as `c-3`.

| Keys | Effect |
| --- | --- |
| `Ctrl+N` `3`, type `5`, `Enter` | Push 5 three times |
| `Ctrl+N` `3` `Enter`, nothing typed | Push three more copies of the top item |
| `Ctrl+N` `3` `Tab` | Move the item at level 3 to the top: `1 2 3 4` becomes `1 3 4 2`. With 3 or fewer items it swaps the top two instead |

## History

RPN grew out of 1920s logic, reached computers between the 1940s and 1960s, and was popularized by Hewlett-Packard calculators from 1968 on.

| Year | Milestone |
| --- | --- |
| 1924 | Polish logician Jan Łukasiewicz invents prefix ("Polish") notation, which writes logic without parentheses. |
| 1941 | Konrad Zuse's Z3 computer takes the operands first and the operation last, an early form of RPN without a stack. His 1945 Z4 adds a two-level stack. |
| 1954 | Arthur Burks, Don Warren and Jesse Wright propose the reversed, postfix scheme. |
| 1957 | Charles L. Hamblin extends it and builds GEORGE, an RPN-based programming system for the DEUCE computer in Australia. |
| Early 1960s | Friedrich L. Bauer and Edsger W. Dijkstra reinvent it independently to cut memory access by evaluating on a stack. Dijkstra's shunting-yard algorithm converts infix to RPN. |
| 1963 | The English Electric KDF9 and Burroughs B5000 computers ship, both built around a stack. Friden's EC-130 brings RPN to desktop calculators with a four-level stack. |
| 1968 | Hewlett-Packard's 9100A desktop calculator uses RPN with a three-level stack. |
| 1972 | The HP-35, the first handheld scientific calculator, introduces the classic four-level RPN stack. |
| 1986 | HP's RPL extends RPN with an unlimited stack that holds symbols, lists and programs as well as numbers. |

RPN also lives on in software: the Unix `dc` calculator and stack languages such as Forth and PostScript use it. rpn-calc's stack, like RPL's, holds more than numbers: expressions and variables live on it too.

Source: [Reverse Polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation), Wikipedia.
