# My personal implementation of PNGme

This repo hosts the code of my attempt at tackling the [PNGme](https://picklenerd.github.io/pngme_book/) exercise by [picklenerd](https://twitter.com/picklenrd), as a way to further improve my fluency in Rust.

Since many unit tests are already provided, I've decided to follow a proper Test Driven Development approach.

## Progress

**Chapter 1: Done**

**Chapter 2: Done**

**Chapter 3: Done**

Chapter 4: In progress

## Future improvements

Currently, many tests rely on the manipulation of one or more files in order to guarantee that the core functionality of the program works correctly when invoked. Each test also uses the same file names to avoid coming up with new ones, meaning that these files can be read or written by multiple tests at the same time.

The bare minimum workaround to avoid mixing up the content of the files is to run the tests sequentially instead of concurrently, which is done by running them with

```
cargo test -- --test-threads=1
```

However, there seem to be better and cleaner ways which don't even need to meddle with the execution performance. So far I've found the following ones.

### • Checking if the required file already exists

At the start of each test, call a function that checks if there's an already existing file with the name that would be used by that test soon after. If there is, it's probably a leftover from other tests that have failed or panicked, so it can either be deleted and recreated, or its content can be wiped clean.

### • Wrapping the files to control their deletion

By using the [Newtype Pattern](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction) it's possible to wrap another struct around a `std::fs::File` instance. Then, a custom `Drop` implementation could be defined for the wrapper so that the inner file gets deleted automatically once the wrapper gets out of scope. This also works in case of a `panic`, since the `Drop` impl gets always called.

### • Using the OS's native temp files

With the crate [tempfile](https://docs.rs/tempfile/latest/tempfile/index.html) it's possible to easily access the operating system's mechanism to create temporary files that get immediately removed as soon as they're not used anymore by the program.

### • Using an abstraction over files

A `std::io::Cursor<Vec<u8>>` provides an interface which behaves in a way that's similar to files, but its content resides in memory and it's perfect for testing purposes, as it doesn't involve any side effect and it requires no extra cleaning steps.