1. Write a program that calls fork(). Before calling fork(), have the main process access a variable (e.g., x) and set its value to something
(e.g., 100). What value is the variable in the child process? What happens to the variable when both the child and parent change the value of x?

ans: you said fork will create a new program so everyting will be forked into the new program, including the x but memory for each program in address space is different so they both should be different, (after paging and all).

2. Write a program that opens a file (with the open() system call) and then calls fork() to create a new process. Can both the child and parent access the file descriptor returned by open()? What
happens when they are writing to the file concurrently, i.e., at the same time?
ans: we just did that in the recent example, where you reminded me to close the fd in both child an parent because both are different. but i do have a quesiton, even if the fd is different arnt we accessing the one and same file? 

3. Write another program using fork(). The child process should print “hello”; the parent process should print “goodbye”. You should
try to ensure that the child process always prints first; can you do this without calling wait() in the parent?
ans: i can definitely do this with calling wait/waitpid but without that we might need to add delay or someting like that but even that will not 100% insure the child will run before, but ther emust be oterh ways. 

4. Write a program that calls fork() and then calls some form of exec() to run the program /bin/ls. See if you can try all of the
variants of exec(), including (on Linux) execl(), execle(), execlp(), execv(), execvp(), and execvpe(). Why do you think there are so many variants of the same basic call?
ans: this will i try myself. i am confident with execl execlp and execvp. and i will focus on execvpe() specially.


5. Now write a program that uses wait() to wait for the child process to finish in the parent. What does wait() return? What happens if you use wait() in the child?
ans: i have wrrote thos already. and  if we write wail in chila it will try to wait for its child ? (usually it should do that or if kernal find it has not child then the child will exit). this is what i thiink, what will happen? 


6. Write a slight modification of the previous program, this time us- ing waitpid() instead of wait(). When would waitpid() be useful?
ans: already did that lot of times, and yes waitpid is usefull when we know which child we wnat to return to, and wait() will do with any child returning 1st.

7. Write a program that creates a child process, and then in the child closes standard output (STDOUT_FILENO). What happens if the child calls printf() to print some output after closing the descriptor?
ans: ok we already wrote almost this kind of program but we did not call println!() after closing STDOUT_FILENO, what will happen if we do it then? shouldnt simply the println!() will be ignored? 

8. Write a program that creates two children, and connects the stan- dard output of one to the standard input of the other, using the pipe() system call.
ans: this will be easy i will try this too on my own.

---  

Your answers show that you've understood most of the chapter. I'll correct the important gaps without giving you the solutions to the programming exercises—you should still write those yourself.

# Review of your 8 answers

## 1 Variables after `fork()` -> Mostly correct
Both processes initially see `x = 100`. After `fork()`, each process has its own logical memory state, so changing `x` in the child doesn't change the parent's `x`, and vice versa.
Your mention of paging is related to implementation: Linux typically uses copy-on-write, so physical memory pages needn't be copied immediately.

## 2 File descriptors after `fork()` -> Important distinction: Yes, both processes can access the file through their inherited descriptors. And yes, they're accessing the same underlying file.
The distinction is:

  - The parent and child have separate descriptor tables.
  - Their corresponding descriptors initially refer to the same open file description.
  - That open file description tracks things such as the current file offset.

Consequently, concurrent writes can affect the same file, and the shared offset can matter. The result depends on the write operations, their sizes, the file type, and synchronization. Separate descriptor tables do not mean separate files.

Closing FD 3 in the child doesn't close FD 3 in the parent. Each process must close its own descriptor when finished.

Yes, the kernel manages the actual file operations, but it doesn't automatically guarantee that your program's intended ordering is preserved. And yes, concurrent writes can create race conditions.
Let's use your `fork()` example.
 ### 1. What happens when both processes write?

Suppose the parent and child inherited the same open file description after `fork()`.
Shared open file description

File offset: 0
Parent
Writes `AAAA`
Child
Writes `BBBB`

The kernel handles both requests, but which write happens first can depend on scheduling.
For ordinary successful writes to a regular file using the shared offset, the kernel updates that offset as part of the write operation. You might get `AAAABBBB` or `BBBBAAAA`.
For larger or multiple writes, don't assume your entire logical message will be kept together with another process's message. A program's sequence of operations is not automatically one indivisible transaction.
## 2. How do you prevent race conditions?

It depends on what you need to guarantee.

| Approach                    | When it's useful                                                                 |
| --------------------------- | -------------------------------------------------------------------------------- |
| `waitpid()`                 | Make the parent wait for the child before continuing.                            |
| `flock()` / `fcntl()` locks | Coordinate cooperating processes accessing the same file.                        |
| A pipe                      | Send data to one process that performs the writes in order.                      |
| `O_APPEND`                  | Append each write at the file's end rather than using the shared current offset. |

Important: `O_APPEND` helps prevent offset-related conflicts for individual writes on local Linux filesystems, but it doesn't make a series of writes into one transaction.
For a beginner exercise, the simplest solution is often to have one process write the file while the other waits.
One question to keep in mind as you experiment: Do you want to prevent both processes from writing at the same time, or do you specifically need to guarantee the order of their writes? Those are different requirements, and the right synchronization mechanism depends on which one you need.


## 3 Make the child print first without `wait()` -> Good reasoning
Correct: a delay is not a guarantee. The scheduler can still run the parent first.
Think about synchronization rather than timing. What could the parent wait for besides the child's termination? You already know a syscall that can help two processes communicate.

#### Hint: consider `pipe()`.

## 4 Variants of `exec()` -> Ready to implement
Good. Try the variants yourself. The central reason for the variants is convenience: they differ in how arguments, environment variables, and executable lookup through `PATH` are provided.
One distinction to remember: `execvpe()` is a GNU extension on Linux, rather than a portable POSIX interface. Its explicit environment and `PATH` lookup behavior are worth investigating.

## 5 What happens when the child calls `wait()`? -> One correction
Your intuition is right that a child can wait for its own children. But it cannot wait for its parent.
If the child has no eligible child process to wait for, `wait()` returns `-1` and sets `errno` to `ECHILD`. It doesn't automatically exit just because it has no children.
Also, `wait()` returns the PID of the child it reaped on success, or `-1` on failure. The child's exit status is reported through the status pointer.

## 6 Why use `waitpid()`? -> Correct

Exactly. `wait()` waits for any eligible child; `waitpid(pid, ...)` lets you target a particular child, and its options can change whether it blocks or which child states it reports.
Tiny correction: it doesn't simply return
