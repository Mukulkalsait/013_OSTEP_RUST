

#### 3. What Exactly is Happening with the Kernel Stack Switch (and Why)?
> Que: Are we copy-pasting or cloning the user stack into the kernel space? -> [No. ABSOLUTELY NOT]

  * RSP in x86 actually holds: [a memory address] (a 64-bit number pointing to [stack is in RAM]).
  * SP in arm64 ?

  * User Stack Pointer points -> RAM_Area that the user program owns.
  * Kernel Stack Pointer points ->  Different, Isolated region of secure RAM [ONLY THE KERNEL CAN ACCESS].

## Why the hardware must switch it:
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

## What "Swapping" Actually Means:

  * It is an instantaneous hardware swap of a 64-bit address 
  * inside the physical CPU register.
  
  * Imagine the CPU has a hidden internal storage pocket 
  * containing a number (e.g., 0x9000 - the secure Kernel Stack address). 
  * The active stack register currently holds 0x1000 (the User Stack address).
  * The moment a trap occurs, 
    * the CPU hardware instantly drops 0x1000 into a temporary holding slot 
    * forces 0x9000 into the active stack register. 
    * Now, when the kernel executes instructions, 
    * it is safely writing to 0x9000 in secure RAM. 
    * When the kernel is done, 
    * the CPU hardware swaps the user's original 0x1000 back into the register, 
    * and the user program resumes without ever knowing its stack pointer was temporarily changed.







## 3. The IDTR [Interrupt Descriptor Table Register + Vector Table] ## 3 The Old Way - Interrupts

  - ON BOOT, KERNAL
  - 1st -> build a table in memory 
  - contain [exact addresses of kernels security guards] (its [SYSTEM CALL HANDLER FUNCTIONS]).

  - kernel -> CPU hardware: "Hey CPU, here is the map to my guards.
  - If [Trap::Success] => Use Map to find...
  - MAP ==> pre-registered.

#### IDTR [Interrupt Descriptor Table Reg + Vec Table] -> Dedicated register inside CPU. 
>>> IDTR Holds -> Phisical Memory Addr of -> (TRAP TABLE/Interrupt-Vector Table)

  * if Trap::Success -> 

      | CPU | Trap Instruction |  
      | --- | --- |
      | x86_64 | SYS-CAll |
      | ARM64 | SVC | 

  * CPU -> IDTR -> Memory Address (Per/REG Kernel::FN::Addr of Sys_CALL) ->
  * flip [CPL/EL] from UserMode => KernelMode ->
  * Jump to [IP/PC] InstructionPointer/ProgramCounter -> 
  * VIA => Safe Kernel entry point.

##### Historically, when a program wanted to talk to the kernel, it used the IDTR (Interrupt Descriptor Table Register).

  * IDTR points to a massive table in RAM (the Trap Table).
  * if TRAP::SUCCESS
  * CPU -> goto physical RAM -> Search MASSIVE TABLE -> find ROW  -> read address => JUMP.

  * PROBLEM: [Going out to RAM] is INCREDIBLY SLOW 
  * for a modern CPU. 
  * Doing this for every single system call created a massive performance bottleneck.


## 4. Hardware System Call MSRs [(Model-Specific Registers)]
  - AMD and Intel introduced [DEDICATED FAST SYSTEM CALL INSTRUCTIONS] (sys-call / sys-ret).

#### The Modern Way - Fast Syscalls
##### MSRs, (Modle specific Reg) ->  <ins>[massive upgrade over the IDTR mechanism.]<ins>

  * IDTR -> Slower
  * MSRs -> Faster (Model-Specific Register)

 >>> x86_64 -> AMD+Intel -> Dedicated Fast SysCall Instruction -> SysCall/Sys-Ret

  CPU => Dedicated Iternal Hw-REg [MSRs] specifically [{IA32_LSTAR}].
  * Boot -> kernel write 'Syscall Handler Entry Addr ->into-> [MSRs]'
  * proc => syscall ->
      - ⛔ memory lookup => Slower
      - ✅ MSRs => SuperFast
  * MSR-> Straignt into Instruction Pointer.

* To fix speed => CPU manufacturers added specialized MSRs directly onto the CPU silicon.
*  created a new instruction -> [SYSCALL].
* Instead of looking up a giant table in RAM -> via the IDTR,
* the kernel writes the address of -> [ main system call handler ] directly into a hardware [MSR] register ON BOOT.


| Feature | IDTR Method (Interrupts) | MSR Method (Syscall) |
|---|---|---|
| Where is the address? | In a table located in RAM | Baked directly inside a CPU Register |
| Speed | Slow (requires memory fetches) | Instantaneous (measured in CPU clock cycles) |
| Primary Use | Hardware interrupts (keyboard clicks, mouse moves, timer ticks) | Software system calls (printf, read, write) |


* MODERN OPERATING SYSTEMS USE BOTH: 
    -  IDTR to handle unpredictable hardware events (like a hard drive finishing a read)
    -  but they use the ultra-fast MSRs for standard software system calls.

#### MSRs vs. TSS: Why MSRs are so much faster

   1. The Old/Interrupted Way (TSS):
      - When TRAP::SUCCESS:
        * TRAVEL : |CPU -> Memory Bus -> RAM |
        * to fetch the kernel stack pointer from the TSS in RAM. 
        * Crossing this => SLOW => physically Far Distance.
   2. The Modern Way (MSRs/Banked Registers): 
      - crossing boundary on every single system call => SLOW.
      - In ARM64: NO-TSS -> 💚 [SP_EL1] register directly inside the CPU silicon.
      - modern x86_64 => caching mechanisms => L1 /dedicated internal reg-> [Rare Boundry Cross happens]

## 5. Control Registers (CR0, CR3, CR4 in x86)
Apart from regular data registers (RAX, RBX), the processor has physical control registers that dictate how the hardware behaves.

* CR0: Contains a physical hardware bit that turns on/off protected mode and memory paging entirely.
* CR3 (The Page Directory Base Register): This is the register that points to the current process's page table in RAM. As you read further into OSTEP's memory management, you will learn that every time the OS performs a context switch, it changes the address inside this physical register, which instantly alters what memory the CPU is capable of seeing.

------------------------------

#### OTHERS:

##### CR0 => Phisical HW bit _> Paging ON/Off
##### CR3 => Current Processes Page Table Phi-Addr -> Ram  
```rust
/// IMP: CR3 -> Context Switching COmponent
```







# SUMMARY: LIST
---
## REF: [TrapCall](./501_TrapCall.md)
## REF: [Kernal Stack Pointer](./502_KernalStackPointer.md)

  - >SYS-Call == Trap => Toggle to (UserMode 🔁 Kernel Mode)
  - > Privi-Lege Ring  / ARM:[CPSR-CurrentProgramStateReg] => ExecutionLevel + ALU_State_flag. 
  - CSR: Code Segment Reg -> 16bit => last 2 bit (0,1,2,3) CPL determined
  - > CPL: Current Previliged Level (x86_64) = Rings(0,1,2,3) 0->kernel, 1/2 I/O driver , 3 user.
  - > EL : Exception Level (ARM64) = EL(0,1,2) 0-User, 1-kernel, 2-Hyper-V

| CPU | Name | User | Kernel | Other | 
| --- | --- | --- | --- | --- |
| x86_64 | Current Previliged Level | Ring 3 | Ring 0 | Ring 1/2 => IO Drivers |
|  | ==Lower 2 bytes or CSR== | ==total 16bit== | ==last2bit== | ==Val(0,1,2,3)== |
| ARM64 | Exception Levels | EL0 | EL1 | EL2 (Hypervisor) | 
|  | ==+ ALU-StateFlag== | ==Z,C,N,O== |  |[Zero,Carry,Negative,oVerdrive]  | 
  - > Privileged Instruction -> (allowed to keral mode only.)
  - CPU::Hardware_block -> block execution of process at HARDWARE level.
  - SPR/SP:[(Stack Pointer {Register})] Phy_reg -> 64bit Physical-Mem-Addr of Kernel Stack in RAM.
  - KSP(Kernel Stack Pointer) : Secure Stack Frame of Kernel

###### [TSS] is ❌ NOT ==HARDWARE REGISTER== inside the CPU chip 
###### Its an [array of bytes] created by the ==OS Kernel== and stored in [RAM]. 
###### Physical CPU [core] only contains a tiny register -> [TR (Task Register)]
###### [TR] holds => ==OS writes== [pointer pointing to TSS struct in RAM] ON [TR]
### Hence: [TR] can be anything => [RSP]-0,1,2,3 or [ISP]-0to7


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
 - TABLE:
    | CPU | UserProgram Stack Loc Traker |  Kernel Stack Loc Tracker |  REG names|
    | --- | --- | --- | --- |
    | x86_64 | RSP(Registor Stack Poing) | [TSS] Task State Segment|==RSP3==,(⛔NoUse:**RSP1/2**),[RSP0]| 
    | ARM64 | SP (Stack Pointer) |  [SP_EL0/SP_EL1] | ? |

  - TSS: Task state seg [Struct Burn in CPU -> OS Generate RAM ] => Write Loaction in ==TSS::RSP0/RSP3==.
  - CPU STRUCT contain [**(RSP0)** RING_0 StackPointer] 64 bit pointer.
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

  - [CHIP/MEMORY BOUNDRY]: ==traveling of cpu to ram for fetching the Struct==
     * CPU know Struct TSS{} => ram have TSS::new();
     * CPU READ from RSP0 => travel via BUS-Lanes => RAM -> Read | [SLOW].

    | CPU SILICON CHIP | Transport |     EXTERNAL RAM STICKS |
    | --- | --- | --- |
    | [Task Register (TR) ]|   | |
    |  #        |  | |
    |(Holds RAM Address)| | "[TSS Data Structure]" |
    |  #        |  | +---------------------+ |
    | --System Bus wires-->| --Sys-Bus-->|----> Offset 4: [KSP] |
    | |  | +---------------------+ |

  - RST0: Phy-CPU [INCREDIBLY FAST TRANSISTORS] -> Only 64-bit Data => Os store StackPointer of Ring0
  - BOOT SETUP TR:  
    - KERNAL > "[SBM]" (SECURE BOOT MEM) => TSS::new() {Struct Defined in CPU}
  - [IST] => Interrupt Stack Table (O to 7)
  









































































