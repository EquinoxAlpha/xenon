use rhai::Engine;

use crate::hwbp::{HardwareBreakpoint, HardwareBreakpointType};

use super::{Context, RuntimeCallback};
pub fn register_functions(engine: &mut Engine, context: Context) {
    let ctx = context.clone();
    engine.register_fn("breakpoint", move |addr: i64, callback: rhai::FnPtr| {
        let breakpoint = HardwareBreakpoint::new(addr as _, HardwareBreakpointType::Execute, 1).unwrap();
        let callback = RuntimeCallback::Breakpoint(breakpoint.dr, callback);
        ctx.debugger().callbacks.push(callback);
        ctx.debugger().breakpoints.push(breakpoint);
    });

    let ctx = context.clone();
    engine.register_fn("watchpoint", move |addr: i64, length: i64, callback: rhai::FnPtr| {
        let breakpoint = HardwareBreakpoint::new(addr as _, HardwareBreakpointType::Access, length as _).unwrap();
        let callback = RuntimeCallback::Breakpoint(breakpoint.dr, callback);
        ctx.debugger().callbacks.push(callback);
        ctx.debugger().breakpoints.push(breakpoint);
    });
}
