# Xenon
A scriptable debugger

This debugger is programmed using a script, as opposed to a more traditional interactive interface. The intended use case is monitoring or patching highly dynamic environments (without modifying the target binary), where targeted functions can be called hundreds of times per second. It currently supports hardware breakpoints on x86 (testing needed) and x86_64. Linux-only as of right now.

Can currently only attach to existing processes.

Check `script.rhai` for more information, and a simple example.

## Running
```
cargo build --release
sudo ./target/release/xenon [Process ID] [Path to script]
```
The debugger watches the script for modifications, and will automatically reload it once the file is modified.

## To do list
* Documentation, proper README
  - For now, check `src/runtime/` for a complete list of all functions that are callable from scripts
* More testing on multithreaded targets
* Better argument parsing
* Callbacks on thread start/exit
* Software breakpoints
  - Hardware breakpoints (and watchpoints) are on x86 and x86_64 limited to a total of 4.
* Split the debugger and script runtimes into separate libraries 
* Write a DSL
  - Rhai as a general-purpose scripting language is fine, but for this use case has some problems

...and probably more, not in that order.

## Known issues
* There's a small chance that adding a breakpoint will cause an error while writing to the debug registers. I have not investigated the issue much and do not know why.
* Reloading the script while it is asking for input may cause a crash for an unknown reason.  

Note/warning: The sample target in `test` makes GET requests to `http://www.google.com/`. Change the URL if that concerns you.

## License
GNU GPLv3.

[Rhai](https://github.com/rhaiscript/rhai), the scripting language used here, is licensed under the MIT license, or Apache-2.0.
