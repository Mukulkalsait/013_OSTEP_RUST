
use libc::dup2;
use std::process;

///
/// > read_write
/// ### stdin (0) ──read──> buffer ──write──> stdout (1)
///
/// | Process Creation & Management | File Descriptor I/O | Programming Environment |
/// | --- | --- | --- |
/// | fork() | open() | execve() |
/// | exec*() | close() | getenv() |
/// | _exit() / exit() | read() | setenv() |
/// | getpid() | write() | chdir() |
/// | getppid() | dup() | getcwd() |
/// | kill() | dup2() | -|
/// | wait() | pipe() | -|
/// | waitpid() | - | - |
/// | signal() / sigaction() | - | - |
///
/// #### Workflow:
///  1. Printing the message with WRITE function (no println)
///  2. reading the standard input and storing inside buf.
///  3. Printing the stored input into console with Write function.
///
pub fn read_write() {
    let msg = b"Type someting and hit Enter.\n";

    unsafe {
        libc::write(libc::STDOUT_FILENO, msg.as_ptr() as _, msg.len()); // Y: 1 

        // GEN:
        // 32 byte | u8 = 1bite | 0 of u8
        // ┌─────────────────────────────┐
        // │ 0 │ 0 │ 0 │ 0 │ ... │ 0 │ 0 │
        // └─────────────────────────────┘
        // read() nead the space to put the data.
        let mut buf = [0u8; 32];

        let _read_resualt = libc::read(libc::STDIN_FILENO, buf.as_mut_ptr() as _, buf.len()); // Y: 2
        // eg:  read(from, put data in, maximum_amount)
        println!("0 Success/ -1 Fail:{}",_read_resualt);
        libc::write(libc::STDOUT_FILENO, buf.as_ptr() as _, _read_resualt as usize); // Y: 3
    }
    println!("-------------------------------------------\n| println == libc::write() | BOTH SAME | .\n-------------------------------------------");
}




/// # Sequence
/// 1. open FILE DESCRIPTOR => fd
/// 2. duplicate it on copy.
/// ### R: make sure to close all the file descriptors once done. 
///
/// eg:
/// ##### DUP FAMILY 
/// 1. let x = **dup(a)** // copy a into x
/// 2. **dup2(a,x)** // copy a into x
/// 3. **dup3(a,x,FLAGS)** // copy a into x + special flags
///     - O_CLOEXEC => close on exec 
pub fn duplicate_fd() {
    let fd = unsafe { libc::open(c"test.txt".as_ptr(), libc::O_CREAT | libc::O_WRONLY, 0o600) };
    // Need to create?        → O_CREAT
    // Need to truncate?      → O_TRUNC
    // Need write access?     → O_WRONLY
    // Need read + write?     → O_RDWR
    // Need append?           → O_APPEND
    let copy = unsafe { libc::dup(fd) };

    eprintln!("original = {}, duplicate: {}", fd, copy);
    unsafe {
        libc::close(fd);
        libc::close(copy);
    }
}



pub fn duplicate_two(){
    let fd = unsafe { libc::open(c"test.txt".as_ptr(), libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC , 0o600) }; // orignal will be 3
    
    let copy = unsafe { libc::dup2( libc::STDOUT_FILENO, fd) }; 
    // Y: copy retunrs the fild descriptor and not the success/fail 
    // (change x -> y) => stdout -> fd so both stdout -> 3 and fd -> 3
    eprintln!("original = {}, copied = {}",fd, copy);
    unsafe {
        libc::close(fd);
        libc::close(copy);
    }
}


/// # fd table
///
/// | No. | name | change | step | fn |
/// | --- | --- | --- | --- | --- |
/// | 1 | stdIn| - | - | -| 
/// | 2 | stdOp| -> output.txt | 3| dup2()|
/// | 2- | output.txt | -> orignal | 5|  dup2()|
/// | 3 | StdErr| - | - | - |
/// | 4 | orignal| **dup_of_2** | 1 | dup() |
/// | 4->X | orignal | REMOVED |  6 | close |
/// | 5 | output.txt| new | 2 | open() |
/// | 5->X | output.txt| REMOVED | 4 | close() |
///
///
/// > Everyting between step 3 and step 5 will go into the file.
pub fn redirect_stdout(){
    let orignal = unsafe { libc::dup(libc::STDOUT_FILENO)}; // Step: 1
    let fd = unsafe { libc::open(c"output.txt".as_ptr(), libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC , 0o600,) }; 
    // Step: 2

    unsafe {
        libc::dup2(fd, libc::STDOUT_FILENO); // Step: 3
        libc::close(fd); // Step: 4
    }
    println!("Every input between step 3 and step 5 will go to the file 'output.txt' including this sentence."); 
    unsafe {
        dup2(orignal, libc::STDOUT_FILENO); // Step: 5
        libc::close(orignal); // Step: 6
    }
    println!("Output Printed to file 'output.txt'");
}

/// 1. fd create,
///
/// 2. fork 
/// 3. in CHILD: 
///     - dup2(fd , stdout) + close FD                              Y: 
///     - execvp with wc command, + libc::_exit(127) on falure
/// 4. in PARENT: 
///     - close FD                                                  Y: 
///     - waitpid
///     - format + wirte.
pub fn redirect_stdout_x(){
    let file_descriptor = unsafe { libc::open(b"xfile.md".as_ptr() as _, libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC , 0o600) }; // fd3

    unsafe {
        let rc = libc::fork();
        if rc<0{process::exit(1)};

        if rc == 0{
            libc::dup2(file_descriptor, libc::STDOUT_FILENO); // stdout => fd
            libc::close(file_descriptor); // fd closed
            
            let cmd = std::ffi::CString::new("wc").unwrap();
            let file = std::ffi::CString::new("p4.output").unwrap();
            let argv = [cmd.as_ptr() as *mut libc::c_char, file.as_ptr() as *mut libc::c_char, std::ptr::null_mut()];
            libc::execvp(cmd.as_ptr(), argv.as_ptr()); // this program will do the work of writing
            eprintln!("execvp faid; libc::_exit(127) triggered.");
            libc::_exit(127); // if fails exit.
        }else{
            libc::close(file_descriptor);
            let mut s = 0;
            let ret = libc::waitpid(rc, &mut s, 0);
            let res = format!("{}\n",ret);
            libc::write(libc::STDOUT_FILENO, res.as_ptr() as _, res.len());
        }
    }
}

///        PIPE
/// ┌───────┬───────┐
/// │ 0 u8  │ 0 u8  │
/// │ FD 3  │ FD 4  │
/// └───────┴───────┘
///    ⬆️      ⬆️
/// ┌───────┬───────┐
/// │ Read  │ Write │
/// │ End   │  end  │
/// └───────┴───────┘
pub fn simple_pipe(){
    let mut fds = [0;2];
    unsafe {
        libc::pipe(fds.as_mut_ptr());

        eprintln!("read fd = {}",fds[0]);
        eprintln!("write fd = {}",fds[1]);

        libc::close(fds[0]);
        libc::close(fds[1]);
    }
}

///           pipe()
///             │
///       ┌─────┴─────┐
///    fds[0]       fds[1]
///     READ         WRITE
///       └─────┬─────┘
///             │
///           fork()
///          /      \
///         /        \
///    PARENT         CHILD
///       │             │
/// close(write)   close(read)
///       │             │
///       │         write(4, msg)
///       │             │
///       │             ▼
///       │          ┌──────┐
///       │          │ PIPE │
///       │          └──┬───┘
///  read(3, buf) ◄─────┘
///       │
///       ▼
///     buf
///       │
/// write(1, buf)
///       │
///       ▼
///    terminal
///
pub fn pipe_communication(){
    let mut fds = [0;2]; // 2 sides of pipe.
    unsafe {
        libc::pipe(fds.as_mut_ptr()); // pipe creation

        let rc = libc::fork(); // parent child seperate
        if rc <0{
            eprintln!("fork faild: linux returns -1");
            process::exit(1);
        }

        if rc == 0 {
            libc::close(fds[0]); // parent file descriptor closed.
            
            let mess = b"Hello from child\n";
            let n = libc::write(fds[1], mess.as_ptr() as _, mess.len()); // sending to parent.
            // (where to write, what to write, length)
            if n == -1 {
                eprintln!("Write faild as linux returns -1");
            }else {
                eprintln!("Write success as linux returns {}n number of bytes",n);
            }
            libc::close(fds[1]); // child closed
            libc::_exit(0); 
            // libc::_exit() => no clenaup, no flush stdin/stdout, no atexit handlers , no user clenaup
            // HENCE: NO RUST drop implimentation run needed. 
            // IMP: used spcficically in child with fork method.
        }else{
            libc::close(fds[1]); // child's file descriptor closed.
            
            let mut buf = [0u8;64]; // resistor size to store data.
            let n = libc::read(fds[0], buf.as_mut_ptr() as _, buf.len());

            // Y: Eprintln vs libc::write() imlimentation. 
            eprintln!("EPRINGLN==> parent recieved string '{:?}' with '{} n bytes'.",buf,n );
            eprintln!("  This down is using libc::write function, While this   above is rusts epringln statements.");

            libc::write(libc::STDOUT_FILENO, buf.as_ptr() as _ , buf.len());

            libc::close(fds[0]);
            libc::waitpid(rc, std::ptr::null_mut(), 0);
        }
    }
}

