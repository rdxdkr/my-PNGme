# My personal implementation of PNGme

This repo hosts the code of my attempt at tackling the [PNGme](https://picklenerd.github.io/pngme_book/) exercise by [picklenerd](https://twitter.com/picklenrd), as a way to further improve my fluency in Rust.

Since many unit tests are already provided, I've decided to follow a proper Test Driven Development approach.

## Roadmap

The project can be considered complete according to the requirements of the exercise, but many tests I've written rely on the manipulation of one or more files in order to guarantee that the core functionality of the program works correctly. Each test also uses the same file names because I wanted to keep things "simple", meaning that these files can be read or written by multiple tests at the same time.

The bare minimum workaround to avoid mixing up the content of the files is to run the tests sequentially instead of concurrently, which is done by running them with

```
cargo test -- --test-threads=1
```

I've been thinking about solutions which could allow for an easier testing experience without altering the present structure of the project. However, I recently realized they were all flawed approaches, meaning that deeper changes are required to properly fix all the current shortcomings. Here's a list of what I'll be working on in the near future:

- [ ] Create a strong separation between library code (core functionality, PNG manipulation) and application code (CLI, handling IO)
- [ ] Use `clap` in a more idiomatic way by accepting the needed types instead of `String`
- [ ] Get rid of every `unwrap()` in the code, checking each possible error source with tests
- [ ] Improve error types and messages if needed
- [ ] Provide a better strategy for testing the application functionality on files, with proper setup and teardown

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)