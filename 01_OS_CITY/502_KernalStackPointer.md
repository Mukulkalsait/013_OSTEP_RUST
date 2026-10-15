
## 2. The [Kernel Stack Pointer] (KSP)

#### KERNAL STACK POINTER:
  * Process => [stack in memory] ->  tracked by a [Stack Pointer register] ([SPR] in x86, [SP] in ARM)
  * RSP /SP are Phisical Reg in CPU => 64 bit memory addr only.
  * <u>WHILE: CPU maintained [separate Stack Pointer] specifically for Kernel Mode.<u> ==> Kernel Stack Pointer.
  * When [TRAP::]SYS_CALL->
    - Program Stack (Os Crash/Corruption) ⛔ (NO)
    - Kernel Stack 🟩 (OK)

> NOT A REAL CODE.
```rust (RUST STYLE)
/// Previliged Checking.
fn Previliged_work(...){
  if Privileged_Instruction {
    if CPL_or_EL(Ring 0 || EL1){ // G: KERNEL Mode
      OK()
    }else{ 
      CPU::hardware_block || process::no_execution ;
      eprintln("Exception");
    }
  } 
}

/// TRAP Mechanism..
if TRAP::SYS_CALL{
  CPU_HW::Csr::Write::CPL::Ring_0; // x86_64 CPL upgrade
  CPU_HW::Cpsr::Write::EL::EL_1 // ARM64 EL upgarde

  Active_Stack_Ptr::Switch {
    User_Stack => Kernel Stack;
  }
  program::Previliged_work(...)
} 
/// NOT A REAL CODE
```


| CPU | UserProgram Stack Loc Traker |  Kernel Stack Loc Tracker |  extra |
| --- | --- | --- | --- |
| x86_64 | RSP(Registor Stack Poing) | [TSS] Task State Segment| ? |
| ARM64 | SP (Stack Pointer) |  [SP_EL0/SP_EL1] | ? |


## [OLD] x86_64: Task State Segment (TSS) : [HYBRID ENTITY]
#### modern x86_64 processors:
  > CPU dedicated hardware structure => TSS () =>[HYBRID ENTITY] Intel/AMD's architecture
  >> CPU hardware has layouts BLUE_PRINT. 
  >>> OS: Generate Data in RAM : From BLUE PRINT 
  ##### Hence Cpu => DirectRead

  - Basically (Rust style understanding)
    ```rust
    import CPU::RSP0;
    Struct TSS{ ... } 

    // IN RAM -> Secure block 
    #[Memory::SecureBootMemory]
    let Ring0 = TSS::BluePrint::new(...)  // creating new Ring0 in ram
    CPU::TaskStateSeg::RSP0::Physical_Hardware_Write(&Ring0); // writing location on RSP0 

    #[Memory::GeneralMemory]
    let Ring3 = TSS::BluePrint::new(...)
    CPU::TaskStateSeg::RSP3::Physical_Hardware_Write(&Ring3);
    // NOT A REAL CODE: JUST UNDERSTANDING
    ```

    ``` Text
      * TSS Hardcoded in Silicon: 
        - TSS is completely fixed by Intel and AMD inside the CPU’s hardware logic.
        - CPU physically Know RSP0 [Kernel Stack Pointer's] exact byte offset 
        - bytes 4 to 11 in 64-bit mode within that structure.
      * The Allocation is done OS: During boot 
        - kernel allocate small, secure block of raw bytes in the system's actual RAM sticks.
        - FORMATS match blueprint => RSP0 slot.
      * The Connection: kernel > phy-mem-addr of block in RAM ==> the CPU's internal Task Register (TR).
    ```

## [New] x86-64: IST Stacks (IST1 through IST7)

  - In 64-bit mode
  - x86-64 introduced a powerful upgrade
  - Interrupt Stack Table (IST)
    - COLLECTION OF 7 "[stack pointer fields]" ==SPF== inside the [TSS]:
    - IST1, IST2, IST3, IST4, IST5, IST6, IST7

    | Tech | Usage | Other |
    | - | - | - |
    | TSS | [ONLY] changing privilege Ring 0 or 3, (1,2)rare| |
    | IST | can be forced into action even when CPU = Ring 0 | | 


##### IST : OS can add : High Priority Interrupts [Total Sys falure Prevention.] 
  - OS config => high-priority interrupts 
  - use [IST] stack ***Prevention** total system failure* :
  - Famous use cases:
    1. Double Fault Handler (#DF): 
       - If the kernel runs out of regular stack space and crashes, 
       - it triggers a Double Fault. 
       - If it tried to handle this using the regular stack, 
       - it would crash instantly again 
       - (a Triple Fault, causing a hardware reboot).
     + By switching to a clean IST stack, the OS can safely report the error.
    2. Non-Maskable Interrupts (NMI): 
     + Critical hardware failure alerts that must be processed on a guaranteed, clean stack.
    3. Machine Check Exceptions (#MC): 
     + Severe hardware errors (like ECC memory corruption).

| RSP0|TSS|Privilege elevation Ring 3->0|Standard SysCall, Page Faults, regular HW interrupts|
| - | - | - | -  |
|Stack Pointer| Where It's Located|  Trigger Condition|Common Real-World Use|
|RSP1 / RSP2|TSS|Privilege elevation to Ring 1 / Ring 2|Unused by modern OSes|
| IST1 – IST7|TSS-IST)| BluePrint=[IDT](Interrupt Descriptor Table) |Emergency handlers (Double Faults, NMI, Kernel Debugging) |


------------------------------
## 2. What is the "Chip/Memory Boundary"?
 "chip/memory boundary" =>  CPU silicon end -> wires leading out to your external RAM sticks begin.


| CPU SILICON CHIP | Transport |     EXTERNAL RAM STICKS |
| --- | --- | --- |
| [Task Register (TR) ]|   | |
|  #        |  | |
|(Holds RAM Address)| | "[TSS Data Structure]" |
|  #        |  | +---------------------+ |
| --System Bus wires-->| --Sys-Bus-->|----> Offset 4: [KSP] |
| |  | +---------------------+ |


  * Inside CPU => physical Task Register (TR) ==INCREDIBLY FAST==, made of [TRANSISTORS on the chip],
  * only hold a [single 64-bit number] (the memory address pointing to the TSS).

  * On RAM =>  actual TSS Data structure.
  * Crossing the Boundary: if TRAP::SUCCESS 
      - CPU Did not stored the kernel stack pointer inside its own local registers. SO: 
      - CPU look at TR register Data
      - send signal across the physical wires (the system bus) crossing the chip boundary out to the RAM,
      - read the bytes from the TSS structure,
      - and bring that kernel stack address back onto the chip.

--- 
   1. The Setup: BOOT 
      - OS allocates 'SECURE BLOCK OF MEMORY' for the CPU => TSS.
      - TSS Contain: dedicated field called [RSP0 (Ring 0 Stack Pointer)].
      - RSP0 Stores => [secure kernel stack address]
      
    TSS => RSP0 => 64 Bit Secure Kenral Stack Addr.

   2. The Hardware Connection: 
      - The CPU has a specific internal register called the Task Register (TR) 
      - that points directly to this TSS structure on the chip/memory boundary.
   3. The Switch: if TRAP::SUCCESS 
      - CPU hardware automatically reads secure add [RSP0] slot via the Task Register 
      - forces that value 
      - straight into the main physical 
      - RSP register, instantly switching your stack.

## In ARM64: Banked Stack Pointers (SP_EL0 and SP_EL1)
ARM64 handles this in a cleaner, more modern hardware fashion by using banked registers. ARM64 physically builds multiple distinct Stack Pointer registers right into the silicon.

* SP_EL0: The physical stack pointer register reserved for Exception Level 0 (User Mode).
* SP_EL1: The physical stack pointer register reserved for Exception Level 1 (Kernel Mode).

How the Switch Happens in ARM64:
When running a user program, the CPU uses SP_EL0 to track the user stack. The moment a trap (an exception) triggers and the CPU elevates to Exception Level 1 (Kernel Mode), the internal wiring of the CPU automatically disconnects SP_EL0 and hooks up the physical SP_EL1 register instead. The software doesn't have to copy or move anything; the processor hardware physically changes which transistor track it is reading from.
Now that the physical registers and the TSS / Banked Register mechanisms are clear, would you like to explore what exactly happens to the user's registers (like RAX, RBX, etc.) right after this stack switch occurs?


## [TR] TASK REG(Stack Registers) [i5-13500HX] Real Life CPU

==1 Where do TSS, RSP, and IST Live?==
  - both IST (Interrupt Stack Table) 
  - and the RSP0/1/2 
  - [FIELDS] => live completely inside the TSS (Task State Segment) structure.
  - [Pointers] => Live Written on [TR] Task Registors on CPU by OS.
    + TSS is not a hardware register inside the CPU chip. 
    + TSS is [array of bytes] created by the OS Kernel => stored in RAM.
    + The physical CPU core only contains a tiny register called the TR (Task Register)
    + which holds a pointer pointing to where that TSS structure sits in RAM.

==2. Cores vs. Threads vs. Registers (The "OSTEP" Reality)== 
  - Your Intel i5-13500HX has a hybrid architecture [i5-13500HX Spec sheat](https://www.intel.com/content/www/us/en/products/sku/232156/intel-core-i513500hx-processor-24m-cache-up-to-4-70-ghz/specifications.html)
  - 6P-cores: ✅ Hyper-Threading,1 physical P-core = 2 Architectural Threads (Logical Processors).
  - 8E-Cores: ⛔ NO Hyper-Threading.

#### The Golden Rule of OS Dev:
  - To the OS => =="Thread" (Logical Processor) is the actual CPU==
  - single [THREADS] is an independent processor. 
  - Therefore, [ Each THREAD ] has its ==own private set of architectural registers. **TR** (Task Reg) ==
  - Thread 0 and Thread 1 (sharing P-core 1) each have their own unique RIP, RSP, CS, and TR registers.
• Because each thread has its own TR register, every single thread has its own separate TSS structure allocated in RAM by the OS. When Thread 0 handles an interrupt, it uses its own RSP0 pointer; when Thread 1 handles an interrupt, it uses a completely different RSP0.

3. Hardware Layout Diagram

This diagram maps out how components are split between the physical CPU package, a Performance Core, and its two independent Hyper-Threads, showing exactly what is private vs. shared.
```text
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │               INTEL CORE i5-13500HX PROCESSOR (DIE / PACKAGE)                   │
 │                                                                                 │
 │  ┌──────────────────────────────────────────────────────────┐                   │
 │  │ UNCORE / SYSTEM AGENT (PCIe 5.0, Memory Controller)      │                   │
 │  └──────────────────────────────────────────────────────────┘                   │
 │  ┌──────────────────────────────────────────────────────────┐                   │
 │  │ SHARED L3 CACHE (24 MB Smart Cache - Shared by all cores)│                   │
 │  └──────────────────────────────────────────────────────────┘                   │
 │                                                                                 │
 │  ┌───────────────────────────────────────────────────────────────────────────┐  │
 │  │               PHYSICAL PERFORMANCE CORE (P-CORE)                          │  │
 │  │                                                                           │  │
 │  │   ┌───────────────────────────────────────────────────────────────────┐   │  │
 │  │   │ SHARED L2 CACHE (1.25 MB per P-core)                              │   │  │
 │  │   └───────────────────────────────────────────────────────────────────┘   │  │
 │  │   ┌───────────────────────────────────────────────────────────────────┐   │  │
 │  │   │ EXECUTION ENG(ALUs,FPUs,VectorUnits -Executing Instructions)      │   │  │
 │  │   └───────────────────────────────────────────────────────────────────┘   │  │
 │  │                                                                           │  │
 │  │   ┌───────────────────────────────┐   ┌───────────────────────────────┐   │  │
 │  │   │ LOGICAL THREAD 0 (CPU 0)      │   │ LOGICAL THREAD 1 (CPU 1)      │   │  │
 │  │   │                               │   │                               │   │  │
 │  │   │ ┌───────────────────────────┐ │   │ ┌───────────────────────────┐ │   │  │
 │  │   │ │ PRIVATE L1 CACHE          │ │   │ │ PRIVATE L1 CACHE          │ │   │  │
 │  │   │ │ (32KB Inst / 48KB Data)   │ │   │ │ (32KB Inst / 48KB Data)   │ │   │  │
 │  │   │ └───────────────────────────┘ │   │ └───────────────────────────┘ │   │  │
 │  │   │ ┌───────────────────────────┐ │   │ ┌───────────────────────────┐ │   │  │
 │  │   │ │ REGISTERS (State)         │ │   │ │ REGISTERS (State)         │ │   │  │
 │  │   │ │  • RIP  (Instr Pointer)   │ │   │ │  • RIP  (Instr Pointer)   │ │   │  │
 │  │   │ │  • RSP  (Current Stack)   │ │   │ │  • RSP  (Current Stack)   │ │   │  │
 │  │   │ │  • CS   (Holds CPL bits)  │ │   │ │  • CS   (Holds CPL bits)  │ │   │  │
 │  │   │ │  • TR   (Task Register)   │ │   │ │  • TR   (Task Register)   │ │   │  │
 │  │   │ └──────────────┬────────────┘ │   │ └──────────────┬────────────┘ │   │  │
 │  │   └────────────────┼──────────────┘   └────────────────┼──────────────┘   │  │
 │  └────────────────────┼───────────────────────────────────┼──────────────────┘  │
 └───────────────────────┼───────────────────────────────────┼─────────────────────┘
                         │                                   │
                    (Points to)                         (Points to)
                         ▼                                   ▼
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │                        MAIN SYSTEM MEMORY (RAM)                                 │
 │                                                                                 │
 │  ┌─────────────────────────────────────┐ ┌────────────────────────────────────┐ │
 │  │ TSS FOR THREAD 0                    │ │ TSS FOR THREAD 1                   │ │
 │  │  • RSP0 -> [Thread 0 Kernel Stack]  │ │  • RSP0 -> [Thread 1 Kernel Stack] │ │
 │  │  • IST1 -> [Thread 0 NMI Stack]     │ │  • IST1 -> [Thread 1 NMI Stack]    │ │
 │  └─────────────────────────────────────┘ └────────────────────────────────────┘ │
 └─────────────────────────────────────────────────────────────────────────────────┘
Use code with caution.
```
Components Decoded for OSTEP:

1. The Registers (RIP, RSP, CS): These dictate the absolute local state of execution. When an OS performs a Context Switch (a major topic in OSTEP Chapter 4/5), it saves these registers from the CPU into a process structure and loads a new process's values into them.
2. The Task Register (TR): Tells the hardware thread where its corresponding metadata sheet (TSS) is located in RAM.
3. L1 and L2 Caching: Private or semi-private fast storage. OSTEP will discuss concurrency and cache-coherence issues later; notice how Thread 0 and Thread 1 share execution units but have isolated states.
Since you are studying OSTEP, would you like to map out exactly what happens to these registers (RIP, RSP, TR) during a Kernel Traps / System Call transition?


