# Scallop

A scalable probabilistic datalog engine with Rust.

## NOTE

A new version of `scallop` is currently under development!
In addition to all we have in the first version, we will support more beautiful syntax, more language constructs, python binding, integration with PyTorch, REPL, and much more!
Stay tuned while we are working on it. Thanks a lot for your interest in this project!

## Usage

The Scallop system is best integrated inside of the Rust context.
With `scallop! { ... }` macro, you can easily embed a Probablistic Datalog program.

``` rust
use scallop_codegen::scallop;

scallop! {
  VQARSmallExample {
    decl name(Symbol, String).
    decl attr(Symbol, String).
    decl is_a(String, String).
    decl target(Symbol).

    name(A, B) :- name(A, N), is_a(N, B).

    is_a("giraffe", "animal").
    is_a("tiger", "animal").
    is_a("wolf", "animal").

    0.8::name(1, "giraffe");
    0.01::name(1, "tiger");
    0.18::name(1, "wolf").

    0.3::attr(1, "tall").

    target(A) :- name(A, "animal"), attr(A, "tall").
  }
}

fn main() {
  let mut prog = VQARSmallExample::<TopKProbProofs<3>>::new();

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.target().complete() {
    println!("{:?}", elem);
  }
}
```

In a Rust program, one will instantiate a new Datalog program with its execution context,
add input facts, can call `run()` to execute the Datalog program.

## Build from source

Please install [Rust](https://rustup.rs) and use the `nightly` branch (`rustup default nightly`).
After that, please build using

```
$ cargo build --release
```

Make sure you add `--release` to ensure its performance.

Then you can install the compiler `sclc` by using the following command.

```
$ cargo install --path compiler
```
