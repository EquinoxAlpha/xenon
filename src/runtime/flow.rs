/// Control flow manipulation functions

use rhai::Dynamic;

use crate::util;

use super::{Context, RhaiThread};

pub(crate) fn register_functions(engine: &mut rhai::Engine, context: Context) {
    // engine.register_fn("return", move |task: RhaiThread, rax: Option<i64>| {
    //     let Ok(mut regs) = util::ptrace::get_regs(task.pid as _) else {
    //         return;
    //     };
    //     if let Some(rax) = rax {
    //         regs.rax = rax as u64;
    //     }
    //     util::ptrace::set_regs(task.pid as _, &regs).unwrap();
    // });

    engine.register_fn("jump", move |task: RhaiThread, rip: i64| {
        let Ok(mut regs) = util::ptrace::get_regs(task.pid as _) else {
            return;
        };
        regs.rip = rip as u64;
        util::ptrace::set_regs(task.pid as _, &regs).unwrap();
    });
}
