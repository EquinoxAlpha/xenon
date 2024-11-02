
breakpoint(0x00401256, |regs, task| {
    print(`Hit increment`);
});

breakpoint(0x00401256, |regs, task| {
    print(`Hit increment 2`);
});