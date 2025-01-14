use riscv::register::{
    sstatus::{self, Sstatus, SPP},
    scause::{self, Trap, Exception}, stval,
};
use crate::syscall::{syscall, SyscallOperation};

#[repr(C)]
pub struct TrapContext {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    pub fn app_init_context(entry: usize, app_id: usize, sp: usize) -> Self {
        unsafe { sstatus::set_spp(SPP::User) };
        let mut ctx: TrapContext = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        ctx.sstatus = sstatus::read();
        ctx.sepc = entry;
        ctx.sp = sp;
        ctx.tp = app_id;
        ctx
    }
}

extern "C" fn rust_trap_handler(ctx: &mut TrapContext) {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let app_id = ctx.tp;
            match syscall(ctx.a7, ctx.a6, [ctx.a0, ctx.a1, ctx.a2, ctx.a3, ctx.a4, ctx.a5], app_id) {
                SyscallOperation::Return(ans) => {
                    ctx.a0 = ans.code;
                    ctx.a1 = ans.extra;
                    ctx.sepc = ctx.sepc.wrapping_add(4);
                }
                SyscallOperation::Terminate(code) => {
                    println!("[Kernel] Process returned with code {}", code);
                    crate::task::exit_current_and_run_next()
                }
                SyscallOperation::UserPanic(file, line, col, msg) => {
                    let file = file.unwrap_or("<no file>");
                    let msg = msg.unwrap_or("<no message>");
                    println!("[Kernel] User process panicked at '{}', {}:{}:{}", msg, file, line, col);
                    crate::task::exit_current_and_run_next()
                }
                SyscallOperation::Yield => {
                    // println!("[Kernel] Task yielded.");
                    crate::task::suspend_current_and_run_next()
                }
            }
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            panic!("[kernel] PageFault in application, core dumped.");
            // crate::loader::APP_MANAGER.run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            panic!("[kernel] IllegalInstruction in application, core dumped.");
            // crate::loader::APP_MANAGER.run_next_app();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
}

#[naked]
#[link_section = ".text"]
pub unsafe extern "C" fn restore_trap() -> ! {
    asm!(
        // 不再将a0作为参数
        "ld     t0, 31*8(sp)
        ld      t1, 32*8(sp)
        ld      t2, 1*8(sp)
        csrw    sstatus, t0
        csrw    sepc, t1
        csrw    sscratch, t2",
        "la     t3, {app_trap_vec}
        csrw    stvec, t3",
        "ld     x1, 0*8(sp)
        ld      x3, 2*8(sp)
        ld      x4, 3*8(sp)
        ld      x5, 4*8(sp)
        ld      x6, 5*8(sp)
        ld      x7, 6*8(sp)
        ld      x8, 7*8(sp)
        ld      x9, 8*8(sp)
        ld      x10, 9*8(sp)
        ld      x11, 10*8(sp)
        ld      x12, 11*8(sp)
        ld      x13, 12*8(sp)
        ld      x14, 13*8(sp)
        ld      x15, 14*8(sp)
        ld      x16, 15*8(sp)
        ld      x17, 16*8(sp)
        ld      x18, 17*8(sp)
        ld      x19, 18*8(sp)
        ld      x20, 19*8(sp)
        ld      x21, 20*8(sp)
        ld      x22, 21*8(sp)
        ld      x23, 22*8(sp)
        ld      x24, 23*8(sp)
        ld      x25, 24*8(sp)
        ld      x26, 25*8(sp)
        ld      x27, 26*8(sp)
        ld      x28, 27*8(sp)
        ld      x29, 28*8(sp)
        ld      x30, 29*8(sp)
        ld      x31, 30*8(sp)",
        "addi   sp, sp, 33*8",
        "csrrw  sp, sscratch, sp",
        "sret",
        app_trap_vec = sym trap_entry, // Mode: Direct
        options(noreturn)
    )
}

#[naked]
#[link_section = ".text"]
pub unsafe extern "C" fn trap_entry() -> ! {
    asm!(
        ".p2align 2",
        "csrrw  sp, sscratch, sp",
        "addi   sp, sp, -33*8",
        "sd     x1, 0*8(sp)
        sd      x3, 2*8(sp)
        sd      x4, 3*8(sp)
        sd      x5, 4*8(sp)
        sd      x6, 5*8(sp)
        sd      x7, 6*8(sp)
        sd      x8, 7*8(sp)
        sd      x9, 8*8(sp)
        sd      x10, 9*8(sp)
        sd      x11, 10*8(sp)
        sd      x12, 11*8(sp)
        sd      x13, 12*8(sp)
        sd      x14, 13*8(sp)
        sd      x15, 14*8(sp)
        sd      x16, 15*8(sp)
        sd      x17, 16*8(sp)
        sd      x18, 17*8(sp)
        sd      x19, 18*8(sp)
        sd      x20, 19*8(sp)
        sd      x21, 20*8(sp)
        sd      x22, 21*8(sp)
        sd      x23, 22*8(sp)
        sd      x24, 23*8(sp)
        sd      x25, 24*8(sp)
        sd      x26, 25*8(sp)
        sd      x27, 26*8(sp)
        sd      x28, 27*8(sp)
        sd      x29, 28*8(sp)
        sd      x30, 29*8(sp)
        sd      x31, 30*8(sp)",
        "csrr   t0, sstatus
        sd      t0, 31*8(sp)",
        "csrr   t1, sepc
        sd      t1, 32*8(sp)",
        "csrr   t2, sscratch
        sd      t2, 1*8(sp)",
        "la     t3, {kernel_trap_vec}
        csrw    stvec, t3",
        "mv     a0, sp
        call    {trap_handler}",
        // 没有返回值
        "ld      t0, 31*8(sp)
        ld      t1, 32*8(sp)
        ld      t2, 1*8(sp)
        csrw    sstatus, t0
        csrw    sepc, t1
        csrw    sscratch, t2",
        "la     t3, {app_trap_vec}
        csrw    stvec, t3",
        "ld     x1, 0*8(sp)
        ld      x3, 2*8(sp)
        ld      x4, 3*8(sp)
        ld      x5, 4*8(sp)
        ld      x6, 5*8(sp)
        ld      x7, 6*8(sp)
        ld      x8, 7*8(sp)
        ld      x9, 8*8(sp)
        ld      x10, 9*8(sp)
        ld      x11, 10*8(sp)
        ld      x12, 11*8(sp)
        ld      x13, 12*8(sp)
        ld      x14, 13*8(sp)
        ld      x15, 14*8(sp)
        ld      x16, 15*8(sp)
        ld      x17, 16*8(sp)
        ld      x18, 17*8(sp)
        ld      x19, 18*8(sp)
        ld      x20, 19*8(sp)
        ld      x21, 20*8(sp)
        ld      x22, 21*8(sp)
        ld      x23, 22*8(sp)
        ld      x24, 23*8(sp)
        ld      x25, 24*8(sp)
        ld      x26, 25*8(sp)
        ld      x27, 26*8(sp)
        ld      x28, 27*8(sp)
        ld      x29, 28*8(sp)
        ld      x30, 29*8(sp)
        ld      x31, 30*8(sp)",
        "addi   sp, sp, 33*8",
        "csrrw  sp, sscratch, sp",
        "sret",
        trap_handler = sym rust_trap_handler,
        kernel_trap_vec = sym super::kernel_trap::kernel_trap, // Mode: Direct
        app_trap_vec = sym trap_entry, // Mode: Direct
        options(noreturn)
    )
}
