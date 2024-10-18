// Sample script for the debug target in `./debuggee`
// Run with `cargo run -- $(pgrep debuggee_binary)`
// The debugger watches this file for changes. Edit the file and save to recompile and reload the script.
//   - Any breakpoints previously set will be automatically removed.
// The scripting language itself is Rhai. You can find the documentation at https://rhai.rs/book.

// You can set an unlimited number of software breakpoints.
// There is a race condition that can occur if a function is called multiple times at once from different threads.
//   - In practice, this only means that the breakpoint hit might not be detected.
// It will not be fixed due to performance reasons and the relatively low impact.
breakpoint(0x401146, |regs, task| { // Entry of shared_func - yours might be at a different address
    print(`Hit shared_func breakpoint from task ${task.pid}`);

    // The `regs` struct contains the registers of the task that hit the breakpoint.
    // It has the same definition as the `user_regs_struct` in `sys/user.h`.

    regs.rdi = 42; // `rdi` is the first argument

    set_regs(task.pid, regs);
});

// On x86 and x86_64, you can set hardware breakpoints.
// These are limited to 4 per task (thread). The debugger applies them to all threads in the debuggee.
//   - Watchpoints (support coming soon!) are actually hardware breakpoints, and therefore also count towards the limit.
// Hardware breakpoints don't suffer from the reliability issues of software breakpoints in this debugger.
hw_breakpoint(0x40115d, |regs, task| { // Return of shared_func - again, yours might be at a different address
    print(`Hit shared_func hardware breakpoint from task ${task.pid}`);

    regs.rax = 1337;

    set_regs(task.pid, regs);
});

// Until proper documentation is written, you can find the list of all available functions in `src/runtime/functions.rs`.