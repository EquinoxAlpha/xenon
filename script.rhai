// Sample script for the debug target in `./debuggee`.
// Run with `cargo run -- $(pgrep debuggee_binary) script.rhai`.
// The debugger watches this file for changes. Edit the file and save to recompile and reload the script.
//   - Any breakpoints previously set will be automatically removed.
// The scripting language itself is Rhai. You can find the documentation at https://rhai.rs/book.

// You can set an unlimited number of software breakpoints.
// These work by placing an `int3` instruction (0xCC) at the break address.
// There is a race condition that can occur if a function is called multiple times at once from different threads.
//   - In practice, this only means that a breakpoint hit might not be detected.
// It will not* be fixed due to performance reasons and the relatively low impact.
// * - a feature flag might be added to enable a workaround in the future
breakpoint(0x401146, |regs, task| { // Entry of shared_func - yours might be at a different address
    print(`Hit shared_func breakpoint from task ${task.pid}`);

    // The `regs` struct contains the registers of the task that hit the breakpoint.
    // It has the same definition as the `user_regs_struct` in `sys/user.h`.

    // `task` is a struct containing information about the task (thread) that hit the breakpoint.

    regs.rdi = 42; // `rdi` is the first argument

    set_regs(task.pid, regs); // optional - you can modify the registers of the task before continuing execution
                              //            keep in mind, you need to call `set_regs(pid, regs)` to apply the changes
});

// On x86 and x86_64, you can set hardware breakpoints.
// These are limited to 4 per thread. As of right now, the debugger applies them to all threads in the debuggee.
//   - Watchpoints (support coming soon!) are actually hardware breakpoints, and therefore also count towards the limit.
// Hardware breakpoints don't suffer from the reliability issues of software breakpoints in this debugger.
// They also have the advantage of not needing to modify the debuggee's memory.
hw_breakpoint(0x40115d, |regs, task| { // Return of shared_func - again, yours might be at a different address
    print(`Hit shared_func hardware breakpoint from task ${task.pid}`);
    print(`Real return value: ${regs.rax}`);

    regs.rax = 1337; // `rax` is the return value

    set_regs(task.pid, regs);
});

// Until proper documentation is written, you can find the list of all available functions in `src/runtime/functions.rs`.