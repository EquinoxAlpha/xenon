// Sample script for the debug target in `./test`.
// Run with `cargo run -- $(pidof test) script.rhai`.
// The debugger watches this file for changes. Edit the file and save to recompile and reload the script.
//   - Any breakpoints previously set will be automatically removed.
// The scripting language itself is called Rhai. You can find the documentation at https://rhai.rs/book.

// TEMPORARY: Please note that this example is for an older version of Xenon, hastily modified for the third(!) rewrite. The addresses are certainly wrong,
//            the debug target has different behavior, and there are no software breakpoints (yet!). Use this example to learn about the basic syntax and usage
//            of Rhai and Xenon's API.

// On x86 and x86_64, you can set hardware breakpoints.
// These are limited to 4 per thread. As of right now, the debugger applies them to all threads in the debuggee.
//   - Watchpoints are actually hardware breakpoints, and therefore also count towards the limit.
// Hardware breakpoints don't suffer from the reliability issues of software breakpoints in this debugger.
// They also have the advantage of not needing to modify the debuggee's memory.
breakpoint(0x401175, |regs, task| { // Return of shared_func - again, yours might be at a different address
    print(`Hit shared_func hardware breakpoint from task ${task.pid}`);
    print(`Real return value: ${regs.rax}`);

    regs.rax = 1337; // `rax` is the return value

    set_regs(task, regs);
});

// Watchpoints are hit whenever the watched value is either read or written to.
// The second argument specifies the length of the target value to watch. Valid values are 1,2,4, and 8 bytes,
// `incrementedEverySecond`, a static variable in the target program, is 4 bytes long, so we write 4 bytes here.
watchpoint(0x404034, 4, |regs, task| {
    print(`"incrementedEverySecond" changed, new value = ${read_i32(0x404034)}`);
});

// Until proper documentation is written, you can find the list of all available functions in `src/runtime/functions.rs`.
// Important functions:
// * read_iX(address) - read X bits of memory as a signed* integer, given a virtual address (valid variants are read_i8, read_i16, read_i32, and read_i64)
// * write_iX(address, value) - same as read_iX, but write data instead
// * read_string(address) - convenience funtion to read a null-terminated string
// * set_regs(pid, regs) - set the registers for the given thread
// * breakpoint(addr, callback), watchpoint(addr, len, callback) - explained above
// Note on the read* functions: if they fail, they will return a (), Rhai's equivalent of "null". Otherwise, they return the read value.
//
// * - Rhai has no unsigned integers.