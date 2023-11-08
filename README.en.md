#### Project Description

This project reverse-engineers the signature algorithm for Apple's caching server and can successfully register as a caching service. The algorithm runs in two modes.

#### Running Modes

1. **Direct Execution (x64)**: Highly efficient but only supports 64-bit CPUs. Tested to work on Windows/Linux/macOS.
2. **Emulator Execution**: Extremely compatible and supports all CPU architectures, including arm64/mips64/riscv64. May be slightly slower.

#### Compilation

- Direct Execution: `cargo build --release`
- Emulator Execution: `cargo build --release --features=emu`

#### Configuration Files

- `cache.json`: Used for setting up IP ranges, similar to macOS options.
- `mac.toml`: Stores machine code information and can be reused on a new Mac machine. Ensure that all five codes are unified.

#### Third-Party Bindings

- Python bindings are supported; you can directly run `register.py`` to use them.
- For `Kotlin/Swift`, please generate the bindings accordingly.

#### Future Plans

1. Expose easy-to-use APIs via cxx for other programming languages (e.g., C++).
2. Transcompile the code by simulating the traces to llvm-ir, and then lifting it to C code.