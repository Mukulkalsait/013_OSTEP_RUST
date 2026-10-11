
> CPU L1 cache / TLB do **not** look up a file descriptor and find the file.

The **TLB** is involved much later when the kernel accesses memory, but it has nothing to do with the meaning of `fd = 1`.

---

# 1. What does "lowest unused FD" actually mean?

Every process has a **file descriptor table** maintained by the kernel. process starts with:

```text
FD       Meaning
────────────────────
0        stdin
1        stdout
2        stderr
```

These are conventionally opened already.

Now:

```rust
let fd = unsafe {
    libc::open(...)
};
```

If 0, 1, and 2 are occupied, the kernel gives you:

```text
fd = 3
```

Now:

```text
0 → stdin
1 → stdout
2 → stderr
3 → your_file
```

Open another:

```text
fd = 4
```

Another:

```text
fd = 5
```

So yes, the numbers change — **but that's completely intentional**. program doesn't normally care that "this file is always FD 3."
It does this:

```rust
let fd = open(...);

read(fd, ...);
write(fd, ...);
close(fd);
```

The returned number is simply a **handle** you keep and use.

---

# 2. if you open 100 files?

```text
0 → stdin
1 → stdout
2 → stderr
3 → file A
4 → file B
5 → file C
6 → file D
...
102 → file #100
```

You store the numbers:

```rust
let fd_a = open(...);
let fd_b = open(...);
```

Then:

```rust
read(fd_a, ...);
write(fd_b, ...);
```

You don't need to know that `fd_a == 3`. You just need the value returned by `open()`.

---

# 3. FD numbers are per-process

```text
Process A

0 → stdin
1 → stdout
2 → stderr
3 → file A
------------------------------
Process B

0 → stdin
1 → stdout
2 → stderr
3 → completely different file
```

```text
       Process A                 Process B
       FD table                 FD table

         3                         3
         ↓                         ↓
       file A                    file X
```

The integer `3` only has meaning **inside that process's FD table**.

---

# 4. Now let's modify your OSTEP program

I'd actually implement it in **three stages**, because each teaches something different.

## Version 1 — Just `open()`

Start with this:

```rust
  let fd = unsafe {
      libc::open(
          c"./p4.output".as_ptr(),
          libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
          0o700,
      )
  };

```

```text
child opened p4.output with fd = 3
```

because:

```text
0 → stdin
1 → stdout // NOT CLOSED 
2 → stderr
3 → p4.output
```

---

# 5. Version 2 —  WE CLOSE 1 -> stdout first.


```rust
if rc == 0 {
    unsafe {
        libc::close(libc::STDOUT_FILENO);
    }

    let fd = unsafe {
        libc::open(
            c"./p4.output".as_ptr(),
            libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
            0o700,
        )
    };

    println!("child opened p4.output with fd = {}", fd);
}

```

NOW 
```text
0 → stdin
1 → FREE
2 → stderr
```

```rust
open(...) // Y:  gets the lowest free number:
```

```text
0 → stdin
1 → p4.output
2 → stderr
```

**That's the trick OSTEP is demonstrating.**

---

# 6. But here's something important about your `println!`

Once FD 1 becomes the file:

```rust
println!("hello");
```

is no longer going to your terminal.

It goes into:

```text
p4.output
```

So don't use `println!()` to verify the FD after redirecting stdout.

Instead:

```rust
eprintln!("opened file as fd = {}", fd);
```

because stderr is still FD 2.

So the useful experiment is:

```rust
if rc == 0 {
    unsafe {
        libc::close(libc::STDOUT_FILENO);
    }

    let fd = unsafe {
        libc::open(
            c"./p4.output".as_ptr(),
            libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
            0o700,
        )
    };

    eprintln!("open returned fd = {}", fd);

    process::exit(0);
}
```

You should get:

```text
open returned fd = 1
```


# 8. But there is a better way: `dup2()`

The `close(1) + open()` trick is excellent for learning because it exposes the "lowest unused FD" rule.

But normally, if your intention is:

> "Make stdout point to this file"

you use:

```c
dup2(fd, STDOUT_FILENO);
```

Conceptually:

```text
open()
  ↓
fd = 3

dup2(3, 1)
  ↓

0 → stdin
1 → p4.output
2 → stderr
3 → p4.output
```

Then you can:

```c
close(3);
```

and get:

```text
0 → stdin
1 → p4.output
2 → stderr
```

This is much more explicit.

So eventually your mental model should be:

```text
open()
    ↓
get a new FD
    ↓
dup2(fd, 1)
    ↓
stdout now refers to same open file
    ↓
close(fd)
    ↓
exec()
```

**For this OSTEP example, however, I would keep the `close(1) → open()` version first.** It teaches you why the trick works. Then we'll do the exact same program with `dup2()`, which is the version you should remember for real Unix programming.



----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------
----------------------------------------------

Exactly — your understanding of the FD table is now correct. And you found an important distinction:

> **`close()` returns a status, not the FD number.**

That's why you got:

```text
Closed: 0
opened: 1
```

### `close()`

Linux's C API is essentially:

```c
int close(int fd);
```

The return value means:

```text
 0  → success
-1  → failure
```

So:

```rust
let closed = unsafe {
    libc::close(libc::STDOUT_FILENO)
};
```

means:

```text
close(FD 1)
     ↓
return 0 → successfully closed
```

It does **not** return `1`.

So your output:

```text
Closed: 0
```

means:

> "FD 1 was successfully closed."

Then:

```rust
let fd = unsafe {
    libc::open(...)
};
```

returns the **new file descriptor**.

Because FD 1 is now free:

```text
Before close:

0 → stdin
1 → stdout
2 → stderr
3 → ...

After close(1):

0 → stdin
1 → FREE
2 → stderr

open()

1 → p4.output
```

Therefore:

```text
closed = 0   ← close succeeded
fd     = 1   ← open returned FD 1
```

Your experiment worked perfectly.

---

## Your other understanding is correct

This:

```rust
libc::STDOUT_FILENO
```

is simply the libc constant:

```text
STDOUT_FILENO = 1
```

Likewise:

```text
STDIN_FILENO  = 0
STDOUT_FILENO = 1
STDERR_FILENO = 2
```

So this:

```rust
libc::close(libc::STDOUT_FILENO)
```

is essentially:

```rust
libc::close(1)
```

but using the meaningful constant is much clearer.

---

## Your shell example is exactly right

When you do:

```bash
2>/dev/null
```

you're saying:

```text
FD 2 (stderr)
       ↓
/dev/null
```

And yes, you can redirect stdin too:

```bash
some_program < input.txt
```

which means:

```text
FD 0 (stdin)
       ↓
input.txt
```

Likewise:

```bash
some_program > output.txt
```

means:

```text
FD 1 (stdout)
       ↓
output.txt
```

So the basic Unix convention is:

```text
0 → stdin
1 → stdout
2 → stderr
```

---

# Now the three mysterious arguments

You asked about:

```rust
libc::open(
    c"p4.output".as_ptr(),
    libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
    0o700,
)
```

Think of `open()` as:

```text
open(
    WHAT FILE?,
    HOW SHOULD I OPEN IT?,
    WHAT PERMISSIONS IF I CREATE IT?
)
```

### 1. `O_CREAT`

```text
O_CREAT
```

means:

> If the file doesn't exist, create it.

Without it, opening a nonexistent `p4.output` fails.

---

### 2. `O_WRONLY`

```text
O_WRONLY
```

means:

> Open it for writing only.

Other common modes:

```text
O_RDONLY → read only
O_WRONLY → write only
O_RDWR   → read + write
```

---

### 3. `O_TRUNC`

```text
O_TRUNC
```

means:

> If the file already exists, truncate its contents to zero length.

For example:

```text
p4.output before:

hello
this is old data
something else
```

Opening with:

```text
O_TRUNC
```

makes it:

```text
p4.output:

(empty)
```

Then `wc` writes its output.

---

### Why the `|`?

This:

```rust
libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC
```

is combining **bit flags**.

Think:

```text
O_CREAT   = 0001
O_WRONLY  = 0010
O_TRUNC   = 0100

             |
             ↓

             0111
```

The actual numbers are different; that's just conceptual.

The kernel receives one integer containing all requested options.

---

# And `0o700`?

This is different.

It's the **file permission mode** used when `O_CREAT` actually creates a new file.

Rust:

```rust
0o700
```

is **octal notation**.

Linux permissions:

```text
700
│││
││└── others
│└─── group
└──── owner
```

Each digit is:

```text
4 = read
2 = write
1 = execute
```

Therefore:

```text
7 = 4 + 2 + 1 = read + write + execute
0 = nothing
0 = nothing
```

So:

```text
0o700
```

means:

```text
owner → read + write + execute
group → nothing
other → nothing
```

For this OSTEP example, `0o700` is simply giving the newly created file restrictive permissions.

**Important:** the mode is relevant when `O_CREAT` creates the file; if the file already exists, the existing permissions aren't replaced by `0o700`.

---

## One tiny thing in your code

You wrote:

```rust
let closed = unsafe { libc::close(libc::STDOUT_FILENO) };
println!("Closed: {}", closed);
```

You correctly discovered the problem experimentally: after `close(1)`, **`println!()` uses stdout = FD 1**.

But at that moment you've already closed it.

So this:

```rust
println!("Closed: {}", closed);
```

is attempting to write to the closed stdout.

Your terminal output didn't show that line. That's expected.

Your `eprintln!()` later works because:

```text
stderr = FD 2
```

and you haven't closed FD 2.

That's actually a **very good demonstration of what FD redirection really means**.

---

### Your experiment in one picture

```text
Initially:

0 ──→ stdin
1 ──→ terminal (stdout)
2 ──→ terminal (stderr)


close(1)
   ↓

0 ──→ stdin
1 ──→ FREE
2 ──→ terminal (stderr)


open("p4.output", ...)
   ↓
returns 1

0 ──→ stdin
1 ──→ p4.output
2 ──→ terminal (stderr)


eprintln!("...")
   ↓
terminal


execvp("wc", ...)
   ↓

wc inherits:

0 ──→ stdin
1 ──→ p4.output
2 ──→ terminal
```

So **your code worked exactly as intended**. The `Closed: 0` result was not a bug — it was the success return value from `close()`.
