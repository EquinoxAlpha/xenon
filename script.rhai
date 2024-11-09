breakpoint(0x00401256, |regs, task| {
    print(`Hit increment`);
});