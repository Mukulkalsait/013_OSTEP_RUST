
## 2. The [Kernel Stack Pointer] (KSP) ✅

#### KERNAL STACK POINTER:
  * Process => [stack in memory] ->  tracked by a [Register Stack Pointer]([RSP] in x86, [SP] in ARM)
  * R[SP]:(x86)/SP:(ARM) are Phisical Reg in CPU => 64 bit memory addr only.
  * <u>WHILE: CPU maintained [separate Stack Pointer] specifically for Kernel Mode.<u> ==> Kernel Stack Pointer.
  * When [TRAP::]SYS_CALL->
    - Program Stack (Os Crash/Corruption) ⛔ (NO)
    - Kernel Stack 🟩 (OK)

> NOT A REAL CODE.
```rust
/// --- 1. PRIVILEGE ENFORCEMENT ENGINE (Pure Hardware Gates) ---
fn hardware_privilege_check(instruction: Instruction, current_cpl_or_el: u8) -> Result<(), CpuFault> {
    if instruction.is_privileged() { // G: Privileged Instruction
        if current_cpl_or_el == 3 || current_cpl_or_el == 0 { // R: User MODE
            trigger_hardware_fault(FaultType::GeneralProtectionFault); // Apps dies instantly
            return Err(CpuFault::AccessDenied);
        }
    }
    Ok(()) // Execution allowed if Ring 0 or EL1
}

/// --- 2. THE TRANSITION: HOW x86_64 DOES IT (RAM-driven) ---
fn x86_64_hardware_trap_handler() {
    // A. Read active Task Register (TR) to find out where the TSS is in RAM
    let tss_ram_address = CPU_REGISTERS::TR.read_address();
    
    // B. Fetch the safe Kernel Stack Pointer (RSP0) out of RAM from that TSS structure
    let safe_kernel_stack = RAM::read_memory_64(tss_ram_address + RSP0_OFFSET);
    
    // C. Save the caller's old state onto the clean kernel stack before overwriting them
    let old_user_rsp = CPU_REGISTERS::RSP;
    let old_user_rip = CPU_REGISTERS::RIP;
    let old_user_cs  = CPU_REGISTERS::CS; // Contains CPL = 3
    
    // D. Update registers to Ring 0 state
    CPU_REGISTERS::RSP = safe_kernel_stack;
    CPU_REGISTERS::CS.set_lowest_2_bits(0); // Hardware switches CPL to Ring 0!
    
    // E. Push old values onto the new kernel stack so we can return later
    push_to_stack(old_user_cs);
    push_to_stack(old_user_rsp);
    push_to_stack(old_user_rip);
    
    // F. Jump to Kernel Entry Point code
    CPU_REGISTERS::RIP = KERNEL::ENTRY_POINT; 
}

// --- 3. THE TRANSITION: HOW ARM64 DOES IT (Wire/Transistor-driven) ---
fn arm64_hardware_trap_handler() {
    // A. Transistor switch flips Current Exception Level inside the CPSR register
    CPU_REGISTERS::CPSR::EL = 1; // Elevated to EL1 (Kernel)
    
    // B. NO RAM READ. The internal selector multiplexer disconnects the 'SP' line 
    //    from the EL0 transistor bank and maps it to the EL1 transistor bank.
    //    SP now magically reflects SP_EL1 without moving a single byte of data.
    
    // C. Save old User Program Counter (RIP equivalent) into a dedicated hardware register
    CPU_REGISTERS::ELR_EL1 = CPU_REGISTERS::PC; // Exception Link Register
    
    // D. Jump directly to vector table
    CPU_REGISTERS::PC = KERNEL::VECTOR_TABLE_EL1;
}
// NOT A REAL CODE:  JUST IMAGINATION/ Understanding
```


  | CPU | UserProgram Stack Loc Traker |  Kernel Stack Loc Tracker |  REG names|
  | --- | --- | --- | --- |
  | x86_64 | RSP(Registor Stack Poing) | [TSS] Task State Segment|==RSP3==,(⛔NoUse:**RSP1/2**),[RSP0]| 
  | ARM64 | SP (Stack Pointer) |  [SP_EL0/SP_EL1] | ? |
  | _  | _  | _ | _ |
  | OS | KSP (Kernal Stack Pointer) | SECURE Space in RAM | ? |


## [OLD] x86_64: Task State Segment (TSS) : [HYBRID ENTITY] ✅
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

## [New] x86-64: IST Stacks (IST1 through IST7) ✅

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
## 2. What is the "Chip/Memory Boundary"? ✅ 
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

  * RSP in x86 actually holds: [a memory address] (a 64-bit number pointing to [stack is in RAM]).
  * User Stack Pointer points -> RAM_Area that the user program owns.
  * Kernel Stack Pointer points ->  Different, Isolated region of secure RAM [ONLY THE KERNEL CAN ACCESS].
      
###### Why the hardware must switch it:
  * When -> Trap Success -> Kernel Needs to run code: 
  * Requirement: Own stack storage.
      - local variables, 
      - function arguments, 
      - and—crucially—the saved registers of your frozen user program.
  * If the CPU did not switch the stack pointer register, 
  * the kernel would start saving its [highly sensitive security data directly onto -> USER STACK ] in user RAM.
    If that happened:
      1. The program could crash the OS:
        * If the user program intentionally set its stack pointer register to point to garbage 
        * or a non-existent memory address,
        * the moment the CPU entered the kernel and tried to push data onto it,
        * the CPU would fault and the entire computer would instantly crash (Blue Screen / Kernel Panic).
      2. A Hacker could exploit it:
        * If the kernel wrote its internal data to the user's stack,
        * the user program could read that memory after the system call finished, 
        * stealing kernel secrets or altering values to hijack the OS.

    TSS => RSP0 => 64 Bit Secure Kenral Stack Addr.

   2. The Hardware Connection: 
      - The CPU has a specific internal register called the Task Register (TR) 
      - that points directly to this TSS structure on the chip/memory boundary.
   3. The Switch: if TRAP::SUCCESS 
      - CPU hardware automatically reads secure add [RSP0] slot via the Task Register 
      - forces that value 
      - straight into the main physical 
      - RSP register, instantly switching your stack.

## In ARM64: Banked Stack Pointers (SP_EL0 and SP_EL1) ✅ 
  - ARM64 Much cleaner using [banked registers] 
  - ARM64 physically [builds] multiple distinct Stack Pointer registers right into the [silicon].
    * SP_EL0:  stack pointer register reserved for Exception Level 0 (User Mode).
    * SP_EL1: The physical stack pointer register reserved for Exception Level 1 (Kernel Mode).

##### Switch Machanism:
  - UserProgram => SP_EL0 to track the USER STACK.
  - TRAP triggers -> CPU elevates to Exception Level 1 (Kernel Mode)
  - CPU [INTERNAL WIRING] auto 
     + disconnects SP_EL0  
     + hooks SP_EL1 register 
       - NO COPY or MOVE anything ⛔  
       - HW physically changes which transistor track it is reading from.

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

#### The Golden Rule of OS Dev:  ✅ 
  - To the OS => =="Thread" (Logical Processor) is the actual CPU==
  - single [THREADS] is an independent processor. 
  - Therefore, [Each THREAD] has its ==own private set of architectural registers. **TR** (Task Reg) ==
  - have their own unique 
    + RIP
    + RSP 
    + CS
    + TR 
  - each thread -> own TR register,
  - every single thread -> own separate TSS 
  - When Thread 0 handles an interrupt -> ==it uses its own RSP0 pointer==
  - when Thread 1 handles an interrupt -> ==it uses a completely different RSP0==

3. Hardware Layout Diagram
```text
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │               INTEL CORE i5-13500HX PROCESSOR (DIE / PACKAGE)                   │
 │                                                                                 │
 │        ┌──────────────────────────────────────────────────────────┐             │
 │        │ UNCORE / SYSTEM AGENT (PCIe 5.0, Memory Controller)      │             │
 │        └──────────────────────────────────────────────────────────┘             │
 │       ┌─────────────────────────────────────────────────────────────┐           │
 │       │ SHARED L3 CACHE (24 MB Smart Cache - Shared by all 20 cores)│           │
 │       └─────────────────────────────────────────────────────────────┘           │
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

 1. The Registers (RIP, RSP, CS):
   - Dictate the absolute local state of execution.
   - When an OS performs a Context Switch (a major topic in OSTEP Chapter 4/5),
   - it saves these registers from the CPU into a process structure 
   - loads a new process's values into them.

 2. The Task Register (TR):
   - Tells HW thread: where corresponding MetadataSheet(TSS) is. 
 3. L1 and L2 Caching: 
   - Private or semi-private fast storage.
   - OSTEP will discuss concurrency and cache-coherence issues later; notice how Thread 0 and Thread 1 share execution units but have isolated states.

---


# Uncore / System Agent / Memory Controller

> OLD: Components like
  - memory controller (RAM) 
  - PCIe controller (GPU/SSD) 
  - OUTSIDE CPU separate MOBO chip=> [Northbridge]

> TODAY:
  - [EVERYTHING EVERYWHERE ALL AT ONCE]: == UNICORE or System Agent ==
  - Uncore: 
    + collective name for all the
    + parts of the CPU silicon 
    + that are not part of the core execution units.
  > JOB: CENTRAL TRAFFIC COP 
   + routing data between your 14 cores
   + the L3 cache 
   + the integrated GPU
   + your PCIe lanes
   + system RAM through the built-in Memory Controller.

# E-Core L2 Cache Layout

  - E-cores => DIFFERENTLY  
    - P-cores -> large and independent,
    - P-core -> own dedicated 1.25 MB L2 cache.

 |[P-CORE]|[E-CORE]|
 | - | - |
 |Large|Much Smaller|
 |Indipendent|Clustered of [4]|
 | 6 P cores | [4X2 Cluster] of E-Core|
 | Seperate L2/P-core ==1.25mb== | Seperate L2/Cluster ==2-4mb== |

 - E-cores are physically much smaller
 - 8 E-cores are split into two clusters of 4. 
 - CLUSTER => single, unified L2 cache ( 2MB-4MB) for 4 E-cores. 
 - ⛔ NO Hyper-Threading on E-cores, so 1 E-core = 1 Thread.

# Execution Engine [EE]
 - Execution Engine =>  physical math and logic factory of the core
 - CPU decodes Instruction: eax, ebx, the 
 - sends to EE 
 - EE It contains:
   + ALUs (Arithmetic Logic Units): For integer math (+, -, AND, OR).
   + FPUs / Vector Units (AVX): For floating-point math and complex multi-data matrix operations.
   + AGUs (Address Generation Units): Specialized calculators that only figure out memory addresses.

# The Main Registers Explained

🟥 RIP (Register Instruction Pointer): This is the Program Counter. It holds the memory address of the very next machine code instruction the CPU thread needs to execute.
🟩 RSP (Register Stack Pointer): Points to the top of the current application or kernel stack in memory. It changes every time you call a function (PUSH, POP, CALL, RET).
    ```text 
    1. Why does RSP change on PUSH/POP/CALL/RET?
    > RSP "points to the top of the stack," 
      - top means the current boundary line where active data ends.
      - The stack memory region is allocated at boot time
      - but it behaves like an empty bucket that grows downwards as you drop items into it.

    > When you execute a CALL or a PUSH
      - you add data to the stack
      - To prevent overwriting what is already there
      - CPU automatically subtracts from RSP
      - (moving it down to a fresh, unused memory address) and writes the data there.
    > When you execute a RET or a POP
      - the CPU reads the data at RSP 
      - and automatically adds to RSP (moving it back up, freeing that space).
    > So, RSP always tracks the current tip of the active stack frame,
      - moving up and down continuously as your functions execute.
    ```
🟨 CS (Code Segment): As we discussed, this tracks the current privilege ring (Ring 0 vs Ring 3) in its lowest two bits.
🟦 TR (Task Register): Holds a pointer to the current thread's TSS structure in RAM.

## Part 2: What is an NMI Stack & The IST1-7 List?

> [IST1]: NMI (Non-Maskable Interrupt)
  - cannot be ignored or disabled by OS (unlike regular,OS can temp mute CLI instructions).
  - [HIGH-PRIORITY HARDWARE EMERGENCY ] alert deu to [catastrophic hardware events].
    + RAM-stick detecting uncorrectable data corruption
    + MOBO overheating.
  - can strike at any microsecond
  - even while the kernel is modifying critical stack 
  - Kernel cannot risk using the active stack. 
  - Therefore -> uses a dedicated emergency stack pointer called [IST1].

####  The IST1–7 Mapping in Modern OSes (like Linux)

- The x86-64 hardware leaves the 7 slots open for the OS to define. Linux configures them as follows:
 🔴 IST1 (NMI Stack): Used for handling Non-Maskable Interrupts safely.
 ⚫ IST2 (Double Fault Stack): Used when the CPU crashes while attempting to handle a previous crash. This emergency stack prevents a hardware hard-reboot (Triple Fault).
 🟡 IST3 (Machine Check Stack): Used for severe internal CPU or memory hardware errors.
 🟢 IST4 (Debug Stack): Used by kernel debuggers to set hardware breakpoints without interfering with normal code execution.
 🔵 IST5 (MCE Safe Recovery Stack): Used in newer kernels to safely recover from specific memory errors.
 🟣 IST6 & IST7: Kept reserved or unassigned for future architecture extensions.

- ON boot=> kernel allocates[isolated, dedicated blocks of RAM] for [7 frames] => 7 memory addresses -> in TSS structure.
- GUARANTEED EMERGENCY ZONES that sit ENTIRELY UNTOUCHED [until] a severe hardware crisis forces the CPU to jump to them.
   
#### Part 3: ARM64 Mechanism vs. x86_64 (Banked Registers & Hardware Wiring)

> ARM64 CPU: [Apple's M-series] / flagship [Qualcomm Snapdragon]
  - completely DIFFERENT & much FASTER -> [PRIVILEGE TRAPS]
  - Instead -> ==complex software structure (TSS)== in RAM
  - ARM64 builds the solution entirely into the hardware copper wires via [Banked Registers]
>Banked Registers
  - In silicon register is just a tiny [CLUSTER OF TRANSISTORS].
  - so here (SP)Stack Pointer Register -> physically duplicated in CHIP.
  - You don't just have one SP; you have:
    • SP_EL0 (Stack Pointer for Exception Level 0 - User Mode)
    • SP_EL1 (Stack Pointer for Exception Level 1 - Kernel Mode)

> How the "CPU Wiring" removes Copying/Moving
 - x86 => SYS_CALL(intrupt) => the CPU read RAM => find TSS => the RSP0 value => copy it into the RSP register | SLOW |
 - ARM64 => (app on EL_0) SYS_CALL(intrupt) => HW instantly elevates to EL1 Physically => [an INTERNAL TRANSISTOR SWITCH FLIPS]
 - The assembly code instruction MOV x1, SP doesn't change,
 - [**but the PHYSICAL COPPER WIRE LINES inside the CPU are DISCONNECTED from the SP_EL0 TRANSISTOR BANK** ]
 - [**and INSTANTLY ROUTED to the SP_EL1 TRANSISTOR BANK instead.** ]

 - No data was read from RAM, 
 - and no values were copied or moved. 
 - The CPU simply changed which physical pool of hardware storage the name SP points to.
 - This makes context transitions in ARM64 incredibly efficient.

### Part 4: Can Regular Users Modify These CPU Registers?

No, and this is the entire foundation of Operating System security (OSTEP Chapter 4: Limited Direct Execution).
If a normal user application could write directly to CS or TR, it could simply execute MOV CS, 0 to elevate itself to Ring 0 and completely take over or wipe the machine.

The Hardware Wall

The CPU hardware enforces strict rules based on the current mode:
1. Instructions: Certain assembly instructions are explicitly flagged as Privileged Instructions (e.g., LTR to load the Task Register, or wrmsr to write model-specific registers). If the CPU sees these instructions while the lowest 2 bits of CS are 11 (Ring 3), the CPU refuses to run them and triggers a General Protection Fault (#GP), killing your application instantly.

2. C-Pointers vs. Registers: When you write code in C or C++, pointers allow you to read and write to addresses in RAM. You cannot create a C-pointer that points to a hardware register because registers do not have memory addresses; they live inside the processor core.

To manipulate registers, low-level programmers use inline assembly (like asm volatile ("..." : : );). Even then, you can only modify general-purpose registers (like RAX, RBX). The critical control registers (CS, TR, CR3) can only be modified by the Operating System kernel running in Ring 0 / EL1.

---

Your structural mental models are excellent. You are asking the exact right questions to bridge software logic (like Rust pseudocode) with electrical engineering reality. Let's break this down piece by piece.


###  x86_64 vs. ARM64 Stack Registers: What's the difference?

[***You asked: Doesn't x86_64 have RSP0, RSP1, RSP2, and RSP3 just like ARM64 has SP_EL0 and SP_EL1?
The critical difference is where they physically exist:
• On x86_64: The processor only contains one physical RSP register inside the silicon chip. The names RSP0, RSP1, and RSP2 are just text labels inside a text document (the TSS structure) sitting out in your RAM sticks. The CPU does not have a transistor cluster named RSP0. It must read RAM to figure out what number to overwrite RSP with.
• On ARM64: The processor actually builds multiple distinct stack pointer registers into the silicon chip. SP_EL0 is a physical set of transistors, and SP_EL1 is a completely separate physical set of transistors right next to it.***]

### The Transistor Switch & "Transistor Banks" Explained

*You asked: [Is that Transistor Bank a storage of CPU? Does something like the TSS need to be stored on the CPU itself?]*
 - Yes, a "Transistor Bank" TERM for a register on the CPU.
 - REGISTER => tiny, incredibly fast storage unit made of transistors inside the core.
 - x86_64 => Very Limited. 
 - ARM64 => multiple physical SP registers directly on the chip
 - ⛔ NO TSS structure Needed.
   + Electrical railway track switch:

```text 
            |-> SP_EL0(Physical Storage) : User Stack 👨 
            |
SP WIRES -> |🔘 (Electronic Swich ) / Multiplexer Controlled by SPSR mode
            |
            |-> SP_EL1(Physical Storage) : Kernal Stack 🔱  
```

 
When an app is running, the switch is up, connecting your code to SP_EL0. The moment a svc (syscall) instruction runs, the hardware changes the value inside the CPSR (Current Program Status Register) to EL1. That electrical change flips the switch down. Now, any instruction referencing the stack pointer is routed to the SP_EL1 transistors.
The CPU didn't have to read a TSS structure from RAM to fetch an address; the address of the kernel stack was already sitting right there in the SP_EL1 transistors because the kernel pre-loaded it when the thread was scheduled.

6. ARM64 Core Architecture Diagram

To contrast with your x86_64 layout, here is how a modern flagship ARM64 processor (like an Apple M-series core or Cortex-X core) structures these components. Notice the complete absence of a TSS pointer or Task Register, replaced entirely by duplicated internal registers.

```text
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │                       ARM64 CORTEX-X (Mac M) CORE DIE                           │
 │                                                                                 │
 │  ┌───────────────────────────────────────────────────────────────────────────┐  │
 │  │ CORE EXECUTION PIPELINE (Decoders, ALUs, NEON/SVE Vector Math Units)      │  │
 │  └───────────────────────────────────────────────────────────────────────────┘  │
 │  ┌───────────────────────────────────────────────────────────────────────────┐  │
 │  │ L1 CACHE (Isolated Instruction & Data caches per core)                    │  │
 │  └───────────────────────────────────────────────────────────────────────────┘  │
 │  ┌───────────────────────────────────────────────────────────────────────────┐  │
 │  │ ARCHITECTURAL REGISTERS (The actual physical silicon storage state)       │  │
 │  │                                                                           │  │
 │  │  • PC   (Program Counter - Equivalent to x86 RIP)                         │  │
 │  │  • CPSR (Current Program Status Register - Holds active EL0/EL1 state)    │  │
 │  │                                                                           │  │
 │  │  ┌─────────────────────────────────────────────────────────────────────┐  │  │
 │  │  │ BANKED STACK POINTERS (Physically distinct transistor sets)         │  │  │
 │  │  │                                                                     │  │  │
 │  │  │  • SP_EL0  ───► [ Transistor Bank A: Active when CPSR Mode = EL0 ]  │  │  │
 │  │  │  • SP_EL1  ───► [ Transistor Bank B: Active when CPSR Mode = EL1 ]  │  │  │
 │  │  │  • SP_EL2  ───► [ Transistor Bank C: Active when CPSR Mode = EL2 ]  │  │  │
 │  │  └─────────────────────────────────────────────────────────────────────┘  │  │
 │  │                                                                           │  │
 │  │  ┌─────────────────────────────────────────────────────────────────────┐  │  │
 │  │  │ EXCEPTION LINK REGISTERS (Holds return addresses without RAM Push)  │  │  │
 │  │  │                                                                     │  │  │
 │  │  │  • ELR_EL1 ───► Stores old PC value when trapping from EL0 -> EL1   │  │  │
 │  │  └─────────────────────────────────────────────────────────────────────┘  │  │
 │  └───────────────────────────────────────────────────────────────────────────┘  │
 └──────────────────────────────────────┬──────────────────────────────────────────┘
                                        │ (Bus Interface)
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │                            MAIN SYSTEM MEMORY (RAM)                             │
 │                                                                                 │
 │  ┌──────────────────────────────────┐     ┌──────────────────────────────────┐  │
 │  │ User Application Stack Space     │     │ Kernel Mode Stack Space          │  │
 │  │ (Tracked by address inside)      │     │ (Tracked by address inside)      │  │
 │  │ [           SP_EL0             ] │     │ [           SP_EL1             ] │  │
 │  └──────────────────────────────────┘     └──────────────────────────────────┘  │
 └─────────────────────────────────────────────────────────────────────────────────┘
```

Key Takeaways for OSTEP Chapter 4 & 5 (Context Switching):

• In x86_64: When switching tasks, the OS kernel must continuously update the RSP0 field inside the RAM-based TSS structure for every thread so that the next interrupt hits the correct kernel stack.
• In ARM64: The OS kernel simply executes a standard processor instruction to overwrite the physical SP_EL1 register on the CPU itself when shifting between execution threads.
Would you like to trace how OSTEP's Context Switch routine saves these distinct registers (RSP vs SP_EL0/1) to a process's Thread Control Block (TCB) when pausing one program to run another?
