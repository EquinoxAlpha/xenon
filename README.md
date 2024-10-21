# Xenon
A scriptable debugger

## what?
This debugger is programmed using a script, as opposed to a more traditional interactive interface. It currently supports software and hardware
breakpoints on x86 (testing needed) and x86_64. Linux-only as of right now.

Can currently only attach to existing processes.

Check `script.rhai` for more information, and a simple example.

## running
```
cargo build --release
sudo ./target/release/xenon [Process ID] [Path to script]
```
The debugger watches the script for modifications, and will automatically reload it once the file is modified.

## todo
* documentation, proper README
  - for now, check `src/runtime/functions.rs` for a complete list of all functions that are callable from scripts
* more testing on multithreaded targets
* better argument parsing
* catch keyboard interrupt to cleanly exit
  - exiting via Ctrl+C leaves int3's and hardware breakpoints in the target, causing it to eventually terminate
  - a crude and temporary workaround is to cause a runtime error in the script
* ~~watchpoints~~
* callbacks on thread start/exit

...and probably more, not in that order.

## known issues
* there is a small chance that stopping all tasks will fail while adding any kind of breakpoint, crashing the debugger
  - probably caused by a race condition involving ptrace somewhere?

## license
GNU GPLv3.

[Rhai](https://github.com/rhaiscript/rhai), the scripting language used here, is licensed under the MIT license, or Apache-2.0.
