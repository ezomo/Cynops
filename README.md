# Cynops — A C-to-Brainfuck Compiler

**Cynops** is a **compiler from a C-like language to Brainfuck**.

> ⚠️ Cynops is *C-like*, but **it is not standard C**.  
> It introduces its own constraints and extensions.

---

## Overview

This project aims to compile a language with C-like syntax into Brainfuck.

Cynops is heavily inspired by and partially based on the following projects:

- **c2bf**  
  [https://github.com/iacgm/c2bf](https://github.com/iacgm/c2bf)\

  Some parts of the code are quoted from this project; without it, this project would not have been possible.
  In particular, it was a major insight to discover the existence of the so-called *Single-while-loop* version of the structured programming theorem, which shows that non-structured control flow can be represented by a single `while` loop simulating a program counter.

- **hydrogen.c (Brainfuck interpreter)**  
  https://github.com/rdebath/Brainfuck/blob/master/extras/hydrogen.c  
  A fast and comfortable Brainfuck interpreter that made Cynops practical to develop and test.

You can think of Cynops as an extension of c2bf with **pointers, arrays, functions, and higher-order functions**.

---

## Supported Features

### Basic Features

- **Types**
  - `int` (16-bit integer)
  - Fixed-point arithmetic (addition and subtraction)

- **Variables**
  - Declaration and initialization
  - Type aliases via `typedef`

- **Arrays**
  - Multi-dimensional arrays

- **Pointers**
  - Arbitrary levels of pointers  
    (*Pointer arithmetic is not allowed*)

- **Structs**

- **Functions**
  - Normal function definitions and calls
  - Recursive calls
  - Function pointers
  - Higher-order functions (functions that take function pointers as arguments)

- **Control Flow**
  - Conditionals: `if` / `else`
  - Loops: `while` / `for`
  - Loop control: `break` / `continue`

---

## Extensions and Design Choices

### Arrays Are Treated as Values

In Cynops, arrays are treated as **values**, not as pointers.

```c
int a[3] = {1, 2, 3};
int b[3] = a;
```

As a result, **arrays can be assigned and returned directly**.

```c
// fn(char [6]) -> char [6]
char (fn(char arg[6]))[6] {
  return arg;
}
```

### Pointer Arithmetic Is Forbidden

Pointers can only be used as references; arithmetic on pointers is not allowed.

```c
int a[3] = {1, 2, 3};
int* p = &a[2];  // OK
p = p + 1;       // NG
```

---

## Usage

### Compile and Run

```sh
./run.sh <filename>
```

### Run Test Cases

```sh
./test.sh
```

---

## Demos

- **Othello (Reversi)**  
  https://www.youtube.com/watch?v=i5pTHRw2-z8

- **Screen Rendering Demo**  
  https://www.youtube.com/watch?v=kMH9iaTFzEQ
