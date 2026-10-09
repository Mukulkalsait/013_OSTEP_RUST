// in c:
// struct context {
//   int eip;
//   int esp;
//   int ebx;
//   int ecx;
//   int edx;
//   int esi;
//   int edi;
//   int ebp;
// };
/// Struct: Context : represents CPU RESISTOR STATE.
/// > the data we need to store process-state.🌟🌟🌟🌟
///
struct Context {
    eip: i32,
    esp: i32,
    ebx: i32,
    ecx: i32,
    edx: i32,
    esi: i32,
    edi: i32,
    ebp: i32,
}

enum ProcState {
    Unused,
    Embryo,
    Sleeping,
    Runnable,
    Running,
    Zombie,
}
//Y: USE
//
// match state {
//     ProcState::Runnable => { // scheduler can choose this process }
//     ProcState::Running => { // process is currently executing }
//     ProcState::Sleeping => { // waiting for something }
//     _ => {}
// }

struct Proc {
    mem: *mut u8, // some address in mem
    sz: u32,      // size of memory
    // kstack:  kernal stack associated with process
    kstack: *mut u8, // R: function calls, local var, saved registor, trap info

    state: ProcState,
    pid: i32,

    parent: *mut Proc,
    chan: *mut u8, // sleeping process waiting on eg. 1.io file, 2. network conn.

    // killed: bool , B: more edmotic way
    killed: i32, // 0=> not killed / 1=> killed, (no bool use because c use 0/1 so we use i32)

    ofile: [*mut File; NOFILE], // array of [T,N] => N elements of type T.
    cwd: *mut Inode,            // Current Working DIR

    context: Context,   // CPU CONTEXT.
    tf: *mut TrapFrame, // IMP: TRAP 🌟🌟🌟🌟🌟
}

// IMP: TRAP => transferring controll from RUNNING code to KERNAL.
//
// # why? =>    1. SystemCall => Program ask kernal for it.
//              2. Interruption => Hardware needs it.
//              3. Exception => CPU DETECTS PRLBEM

#[repr(C)]
/// # a snapshot of the CPU registers at the moment a trap happens.
struct TrapFrame {
    edi: u32,
    esi: u32,
    ebp: u32,
    oesp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,

    gs: u16,
    padding1: u16,
    fs: u16,
    padding2: u16,
    es: u16,
    padding3: u16,
    ds: u16,
    padding4: u16,

    trapno: u32,
    err: u32,

    eip: u32,
    cs: u16,
    padding5: u16,

    eflags: u32,

    esp: u32,
    ss: u16,
    padding6: u16,
}
