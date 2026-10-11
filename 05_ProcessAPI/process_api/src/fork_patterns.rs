//fork_patterns.rs
use std::process;

use libc::fork;

///##### SUMMARY
///> This function will 
///   1. print pid
///   2. create libc::fork()
///   3. check if its not faild or process::exit(1)
///   4. in child print process::id() -> child pid
///   5. in Parent side => 
///      - rip child with waitpid()
///      - print childs pid after ripping and own pid.
pub fn fork_() {
    println!(
        " Hello, (pid:{})
----------------------------------------------------------------------------------------------
After calling fork() **EVERYTHING BILLOW EXECUTE 2 TIMES**
1. in parent side. where fork returns 'child pid' (positive value).
2. in child side. where fork return '0'
🟣 we use 'FORK() RETURN value' in if-else to find parent vs child.
----------------------------------------------------------------------------------------------
",
        process::id()
    );

    let rc = unsafe { libc::fork() };
    if rc < 0 {
        eprintln!("fork faild: LINUX RETURNS 'rc = -1' on FALURE");
        process::exit(1);
    } else {
        println!("Forking Success.");
    }

    if rc == 0 {
        println!("RC==0 | its CHILD | Fork_return:{}      | own_PID:{} | No Reaping Needed | ", rc, process::id());
        process::exit(0);
    } else {
        let mut status = 0;
        let riping = unsafe { libc::waitpid(rc, &mut status, 0) }; // R: this blocks the process untill child finishes.
        println!("RC> 0 | its Parent| Fork_return:{} | own_PID:{} | riping = {}   |", rc, process::id(), riping);
        println!("EVERY CHILD MUST BE REAPED.")
    }
}

/// ##### SUMMARY: DOUBLE FORK
/// > IN THIS FUNCTION
/// 1. fork => (x)
/// 2. fork => (x, y)
/// 3. match (x,y)=> parent(x,y), child-1(0,y), child-2(x,0) ,child-3(0,0)
///     - in each part one who create child will rip it so: 
///         * parent will rip -> c1 and c2
///         * c1 will rip c4.
///
/// ### wait vs waitpid
/// | STATE_VALUE Options | Meaning                                 |
/// | ------------ | ------------------------------------------------------------- |
/// | `0`          | Normal blocking wait; wait for child to terminate             |
/// | `WNOHANG`    | Don't block; return immediately if child hasn't changed state |
/// | `WUNTRACED`  | Also report if child was stopped                              |
/// | `WCONTINUED` | Also report if a stopped child was resumed                    |
///
/// ``` rust 
/// let x = unsafe { libc::wait(<STATE_VALUE>) };
/// libc::waitpid(pid, &mut status, <STATE_VALUE> );
/// // eg
/// libc::waitpid(pid, &mut status, 0);
/// libc::waitpid(pid, &mut status, libc::WNOHANG);
/// ```
pub fn double_fork_n_wait() {
    let pid = process::id();

    let x = unsafe { libc::fork() };
    if x < 0 {
        eprint!("Fork faild: Linux returns -1");
        process::exit(0);
    }
    let y = unsafe { libc::fork() };
    if y < 0 {
        eprint!("Fork faild: Linux returns -1");
        process::exit(0);
    }

    // (x,y) -> (0,Y)
    //   🔽      🔽 
    // (x,0) -> (0,0)
    //
    //  Parent ---> child1
    //                
    //  child-2   child 3
    match (x, y) {
        (0, 0) => {
            println!("[C3]:{} | x:{},y:{}", process::id(), x, y);
            process::exit(0);
        },
        (_, 0) => {
            println!("[C2]:{} | x:{},y:{} | C2 did not created anychild so no reaping.", process::id(), x, y);
        },
        (0, _) => {
            let mut status = 0;

            let c3_reaped = unsafe { libc::waitpid(y, &mut status, 0) }; // R: this blocks the process untill child finishes.
            println!("[C1]:{} | x:{},y:{} | C3 dead with waitpid: {}", process::id(), x, y, c3_reaped);
            process::exit(0);
        },
        (_, _) => {
            let mut s1 = 0;
            let mut s2 = 0;
            let c2_reaped = unsafe { libc::waitpid(y, &mut s1, 0) }; // R: this blocks the process untill child finishes.
            let c1_reaped = unsafe { libc::waitpid(x, &mut s2, 0) }; // R: this blocks the process untill child finishes.
            println!("[P1]:{} | x:{},y:{} |\n Parent with PID:{} Riped both c2:{} and then c1:{}", process::id(), x, y, pid, c2_reaped, c1_reaped);
        },
    }
}

/// ##### SUMMARY: 
/// > IN this function: 
///   1. fork 
///   2. in CHILD side: cmd , file , args => libc::execvp() => println("FAILD")
///   3. in PARENT side: waitpid.
///
/// exec-execv-execp-execl 
/// l = line
/// p = path (autofind)
/// v = vector of args
/// e = env 🌟🌟🌟🌟🌟 
pub fn fork_n_execvp(){
    let pid = process::id();
    let rc = unsafe { libc::fork() };
    if rc < 0{
        eprintln!("Fork faild linux return -1");
        process::exit(1);
    }

    if rc ==0 {
        let cmd = std::ffi::CString::new("wc").unwrap();
        let file = std::ffi::CString::new("p3.c").unwrap();
        let args = [cmd.as_ptr() as *mut libc::c_char, file.as_ptr() as *mut libc::c_char, std::ptr::null_mut() ]; // needed all mutable

        let fork_ret =  unsafe { libc::execvp(cmd.as_ptr(), args.as_ptr()) };
        eprintln!("Fork faild : {}",fork_ret);
        process::exit(1);
    }else{
        let mut s = 0;
        let returning_child = unsafe { libc::waitpid(rc,&mut s, 0)};
        println!("child returned:{}, paretn: {}",returning_child, pid);
    }


}

/// ##### SUMMARY: 
/// > This function will 
///
///   1. create fork 
///   2. in CHILD SIDE close the STDOUT_FILENO (1)
///   3. open FD with name p4.output, (o_create,o_wronly,o_trunc, 0o700) permissions.
///         - this FD will acquire the closed 1 STDOUT_FILENO default.
///   4. write (wc, p3.c, p3.c.len()) => write out put of cmd wc on file pc3 into STDOUT
///   5. on PARENT SIDE => libc::waitpud
///
/// SO: AT THE END WE WILL SEE OP OF THE WC P3.C > INTO FILE P4.OUPTUT.
///
/// ```text
/// PARENT
///   |
///   +── fork()
///         |
///         +── CHILD
///         |     |
///         |     +── close(1)
///         |     |
///         |     +── open("p4.output")
///         |     |       |
///         |     |       +── returns 1
///         |     |
///         |     +── execvp("wc", ...)
///         |             |
///         |             +── stdout is still FD 1   
///         |             +── FD 1 → p4.output
///         |
///         +── waitpid()
/// ```
pub fn fork_exec_open_close() {
    let pid = process::id();

    let rc = unsafe { libc::fork() };
    if rc < 0 {
        eprintln!("ForkFail: Linux return -1");
        process::exit(1);
    }

    if rc == 0 {
        let closed = unsafe { libc::close(libc::STDOUT_FILENO) };
        println!("Closed: {}", closed);
        // R: ALL CALLS returns STATUS OF SUCCESS => 0 Success / -1 Fail
        // not the File descriptor and other thigns.
        //
        // Y: Closing is not good it automatically opens the next file descriptor here.
        // so
        // we use libc::dub(x,y)
        // this dubpicatges the x file into y
        // mean we can do libc::dup(rc,libc::STDOUT_FILENO). may be somethign like this.
        // and with this even if we have opend 100 files,
        // no randome file will open at the place of closed one.

        let fd = unsafe { libc::open(c"p4.output".as_ptr(), libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC, 0o700) };
        // B: this will take up that closed FD1 place so we can see fd = 1.

        if fd < 0 {
            eprintln!("OpenFail: Linux return less than 0 (-1)");
            process::exit(1);
        }

        eprintln!("File Descriptor successed. Closed:{} and opened:{}", closed, fd);

        let cmd = std::ffi::CString::new("wc").unwrap();
        let file = std::ffi::CString::new("p3.c").unwrap();
        let args = [cmd.as_ptr() as *mut libc::c_char, file.as_ptr() as *mut libc::c_char, std::ptr::null_mut()];

        unsafe { libc::execvp(cmd.as_ptr(), args.as_ptr()) };
        eprintln!("execvp() Faild");
        process::exit(1);
    } else {
        let mut status = 0;
        let ret = unsafe { libc::waitpid(rc, &mut status, 0) };
        println!("Proceess:{}, -> Child: {} Returned.", pid, ret);
    }
}


