
# SUMMARY: LIST
---
## REF: [TrapCall, Swapping](./501_ConCepts-CPU.md)
## REF: [Kernal Stack + CPU Stracture](./502_K_Stack_n_CPU_Stracture.md)

  - >SYS-Call == Trap => Toggle to (UserMode 🔁 Kernel Mode)
  - Trap - Emergency Liver [🆘 SYS-CALL] -> Adder_Space Freez -> Privileged 🔼 Jump. (Kernal)
  - > x86_64: [TaskReg] | REG(0,3) | IST(1-7) |  Privi-Lege Ring  
      + [CSR]: Code Segment Reg -> 16bit => last 2 bit (0,1,2,3) CPL determined
      + > [CPL]: Current Previliged Level (x86_64) = Rings(0,1,2,3) 0->kernel, 1/2 I/O driver , 3 user.
      + RAM : TSS => Task State Seg.
  - > ARM: [CPSR-CurrentProgramStateReg] => ExecutionLevel + [ASF]:ALU_State_flag. 
      + > [EL] : Exception Level (ARM64) = EL(0,1,2) 0-User, 1-kernel, 2-Hyper-V

| CPU | Name | User | Kernel | Other | 
| --- | --- | --- | --- | --- |
| x86_64 | Current Previliged Level | Ring 3 | Ring 0 | Ring 1/2 => IO Drivers |
|  | ==Lower 2 bytes or CSR== | ==total 16bit== | ==last2bit== | ==Val(0,1,2,3)== |
| ARM64 | Exception Levels | EL0 | EL1 | EL2 (Hypervisor) | 
|  | ==+ ALU-StateFlag== | ==Z,C,N,O== |  |[Zero,Carry,Negative,oVerdrive]  | 
  - > Privileged Instruction -> (Critical Task, I/O access, keral mode only.)
  - CPU::Hardware_block -> block execution of process at HARDWARE level.
  - R[SP]/[SP]:[{Register}(Stack Pointer)] Phy_reg -> 64bit Physical-Mem-Addr of Kernel Stack in RAM.
  - K[SP]/SP(Kernel Stack Pointer) : Secure Stack Frame of Kernel
  
#### ARM 64:
  - [BR] Banked Registors
  - CPU build multi-Stack-Ptr-Reg-into Silicon.
  - TrapSwitch => Elevate EL1 => Elevation changes [CPU Wiring]
    + CpuWiring => Disconnect SP_EL0 -> hook SP_EL1 
    + Wiring change => No softwear COPY Move needed.
    + HW physically Track everyting.

[***You asked: Doesn't x86_64 have RSP0, RSP1, RSP2, and RSP3 just like ARM64 has SP_EL0 and SP_EL1?
The critical difference is where they physically exist:
• On x86_64: The processor only contains one physical RSP register inside the silicon chip. The names RSP0, RSP1, and RSP2 are just text labels inside a text document (the TSS structure) sitting out in your RAM sticks. The CPU does not have a transistor cluster named RSP0. It must read RAM to figure out what number to overwrite RSP with.
• On ARM64: The processor actually builds multiple distinct stack pointer registers into the silicon chip. SP_EL0 is a physical set of transistors, and SP_EL1 is a completely separate physical set of transistors right next to it.***]


```text 
            |-> SP_EL0(Physical Storage) : User Stack 👨 
            |
SP WIRES -> |🔘 (Electronic Swich ) / Multiplexer Controlled by CPSR mode
            |
            |-> SP_EL1(Physical Storage) : Kernal Stack 🔱  
```


###### [TSS] is ❌ NOT ==HARDWARE REGISTER== inside the CPU chip 
###### Its an [array of bytes] created by the ==OS Kernel== and stored in [RAM]. 
###### Physical CPU [core] only contains a tiny register -> [TR (Task Register)]
###### [TR] holds => ==OS writes== [pointer pointing to TSS struct in RAM] ON [TR]
### Hence: [TR] can be anything => [RSP]-0,1,2,3 or [ISP]-0to7

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

 - TABLE:
    | CPU | UserProgram Stack Loc Traker |  Kernel Stack Loc Tracker |  REG names|
    | --- | --- | --- | --- |
    | x86_64 | RSP(Registor Stack Poing) | [TSS] Task State Segment|==RSP3==,(⛔NoUse:**RSP1/2**),[RSP0]| 
    | ARM64 | SP (Stack Pointer) |  [SP_EL0/SP_EL1] | ? |
    | _  | _  | _ | _ |
    | OS | KSP (Kernal Stack Pointer) | SECURE Space in RAM | ? |

  - TSS-[(HYBRID)]: Task state seg [Struct Burn in CPU -> OS Generate RAM ] => Write Loaction in ==TSS::RSP0/RSP3==.
  - CPU STRUCT contain [**(RSP0)** RING_0 StackPointer] 64 bit pointer.
      ```rust
      import CPU::{RSP0,RSP3,TSS};
      Struct TSS{ ... }  // with CPU

      // IN RAM -> Secure block -> IMP: AT Time of BOOT
      #[Memory::SecureBootMemory]
      let Ring0 = TSS::BluePrint::new(...)  // creating new Ring0 in ram
      CPU::TaskStateSeg::RSP0::Physical_Hardware_Write(&Ring0); // writing location on RSP0 

      /// AT time of running programs used -> RING3
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
  - [IST] => Interrupt Stack Table (O to 7) -> OS use High Priority Intruptions.
    + use stack ***Prevention** total system failure*
    + Lives in TSS (ram) use TR Logical CPUs.
    + Emergency Handlers -> Doubld Fault(#DF) , Non-MaskableInterrupt(#nmi), (#MC).

| RSP0|TSS|Privilege elevation Ring 3->0|Standard SysCall, Page Faults, regular HW interrupts|
| - | - | - | -  |
|Stack Pointer| Where It's Located|  Trigger Condition|Common Real-World Use|
|RSP1 / RSP2|TSS|Privilege elevation to Ring 1 / Ring 2|Unused by modern OSes|
| IST1 – IST7|TSS-IST)| BluePrint=[IDT](Interrupt Descriptor Table) |Emergency handlers (Double Faults, NMI, Kernel Debugging) |
  > [ALL 7 ISTS]: Usage:
  > This 7 are [isolated, dedicated blocks of RAM] stored Untouched for Catastrophic Fallback of OS CRS (Current Running Stack).
  ```text 
  🔴 IST1:(NMI Stack) -> Non Maksable Intrupts Safety -> Points Emergency Stack -> Used at Catastrophic HW Events.
  ⚫ IST2:(Double Fault Stack:to Stop Tripple Crash) -> if 
      if CPU Crash := Handeler => Handeling time Crash := (imposible to handle so) -> another EMERGENCY STACK to REBOOT.
  🟡 IST3:(Machine Check Stack) -> Severlal (Cpu & Memory) HW errors.
  🟢 IST4:(Debug Stack) -> Used for Kernal Debuging -> HW breakpoints settings without interfering Normal Codes.
  🔵 IST5:(MCE Safe Recovery Stack) -> NEW Kernals: Specific MEM-Errors Safe Recovery.
  🟣 IST6 & IST7 => RESERVED FOR FUTURE extensions. 
  ```

##### [TR] Task Register: INCREDIBLY FAST  (Transistor.) -> 64bit data/per.
##### At Trap -> CPU read -> TR Values: 
  - Example: 
      - RSP0 -> Ring0 / EL1 Execution-1
      - IST0-7

### 20 CPUS:
 > i5-13500HX -> 14Core/20Threads => Hybrid 6P 8E core CPU
  - 6P => Hyper-Threads      ✅ => 1CPU => 2 Logical Cores.
  - 8E => No-Hyper-Threading ⛔ => 1CPU => 1 Logical Cores.
#### SO for Operating System "Logical Core" => 1 CPU. Starting from 0 to N
  + 20 Logical Core => 20 Threads => 20 CPU => to OS.
  + ==Each Thread -> OWN private Set of **TR**==
    - ⏭️ RIP [Reg Instruction Pointer] ==Program Counter== => Addr of Very Next Machine Code.
    - 🎯 RSP [Reg Stack Pointer] => (Top of Current App / Kernel Stack) -> in RAM.
    - 👑  CS [Code Segment] => Tracks [CPR] at lowest 2 bites
    - 🖥️  TR [Taks Reg] => Point to TSS (Process OR Kernel) in RAM.

  + **ALL ABOVE ARE SEPERATE FOR 1 LOGICAL CPU**
    
 
### [i5-13500HX] Contents
> Unicore =>  
  + [Memory Controller + PCIE Controller]:==Northbridge== + **iGPU**
  + Central Traffic Cop : Routing data in
     + Inner:
       - 14 Cores
       - L3 cache
       - iGPU 
     + Outer
       - PCIE Lanes
       - Memory
       - Others.
> **EE** [Execution Engine]: 
 - ==Math & Logical Factory==
 - CPU Decodes instruction (eax, ebx, ...) Sents to=> [EE] 
 - [EE] contains ⏬ to perform operations:
   + [ALUs] =>  +, -, &, |, etc.
   + [FPUs && VectorUnits AVX] => FloatPoint + Complex MultiDataMatrix (MDM) 
   + [AGUS Addr_GenerationUnit] => Mem-Address Calculator.⭐⭐⭐⭐⭐ 

> Cores 14 => total 20 Threads.
  - 6 Performance Cores.
  - 8 Efficiency Cores.
    |[P-CORE]|[E-CORE]|
    | - | - |
    |Large|Much Smaller|
    |Indipendent|Clustered of [4]|
    | 6 P cores | [4X2 Cluster] of E-Core|
    | Seperate L2/P-core ==1.25mb== | Seperate L2/Cluster ==2-4mb== |

    ```text
    ┌─────────────────────────────────────────────────────────────────────────────────┐
    │               INTEL CORE i5-13500HX PROCESSOR (DIE / PACKAGE)                   │
    │        ┌──────────────────────────────────────────────────────────┐             │
    │        │ UNCORE / SYSTEM AGENT (PCIe 5.0, Memory Controller)      │             │
    │        └──────────────────────────────────────────────────────────┘             │
    │       ┌─────────────────────────────────────────────────────────────┐           │
    │       │ SHARED L3 CACHE (24 MB Smart Cache - Shared by all 20 cores)│           │
    │       └─────────────────────────────────────────────────────────────┘           │
    │  ┌───────────────────────────────────────────────────────────────────────────┐  │
    │  │               PHYSICAL PERFORMANCE CORE (P-CORE)                          │  │
    │  │                                                                           │  │
    │  │   ┌───────────────────────────────────────────────────────────────────┐   │  │
    │  │   │ SHARED L2 CACHE (1.25 MB per P-core)                              │   │  │
    │  │   └───────────────────────────────────────────────────────────────────┘   │  │
    │  │   ┌───────────────────────────────────────────────────────────────────┐   │  │
    │  │   │ EXECUTION ENG(ALUs,FPUs,VectorUnits -Executing Instructions)      │   │  │
    │  │   └───────────────────────────────────────────────────────────────────┘   │  │
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
                        (Points to)                         (Points to)
                            ▼                                   ▼
    ┌─────────────────────────────────────────────────────────────────────────────────┐
    │                        MAIN SYSTEM MEMORY (RAM)                                 │
    │  ┌─────────────────────────────────────┐ ┌────────────────────────────────────┐ │
    │  │ TSS FOR THREAD 0                    │ │ TSS FOR THREAD 1                   │ │
    │  │  • RSP0 -> [Thread 0 Kernel Stack]  │ │  • RSP0 -> [Thread 1 Kernel Stack] │ │
    │  │  • IST1 -> [Thread 0 NMI Stack]     │ │  • IST1 -> [Thread 1 NMI Stack]    │ │
    │  └─────────────────────────────────────┘ └────────────────────────────────────┘ │
    └─────────────────────────────────────────────────────────────────────────────────┘
    Use code with caution.
    ```


## ARM64: 
 - 
 -
  
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



## IDTR: Interrupt Des-criptor Table Reg => [SysCall Handler Fn Addr table] 

> IDTR Holds -> Phisical Memory Addr of -> (TRAP TABLE/Interrupt-Vector Table)
```rust 
if TRAP::SUCCESS {
  todo(...) /// above given details.
   IDTR.as_ref().iter().match(|call| call.execute());
} // just imaginary code.
``` 
  | CPU | Trap Instruction |  
  | --- | --- |
  | x86_64 | SYS-CAll |
  | ARM64 | SVC | 

> Workflow: 
>> x86_64: 
  + SysCall => Current RSP (3) => TTS
    - RSP0 -> Set RSP->3 [CPL Flip]
    - Check [IDTR]:Interrupt Descriptor Table [Registor]=> search CallHandler
    - EntryPoint -> 🛟 work -> exit.
  - Returning
    - Sys-Ret => Current RSP (1) => TTS 
    - RSP3 -> Set RSP->0 [CPL Flip]

## [ CPSR-CurrentProgramStateReg] in ARM64 no TSS noting just CPSR + banked registors + flippers.
>> ARM64: 
  + SysCall => CPSR-CurrentProgramStateReg 
    - SP_EL0 Flip SP_EL1 
    - Auto HW Wiring Flipping.
  + EntryPoint -> 🛟 work -> exit.
  - Sys-Ret => CPSR -> SP_EL [FLIP].


## MSRs -> Udate over IDTR.

 - Specialised + CPU Build (banked) for. => SYS-Config + Performance Monitor + debuging.
 - NO NAMING -> 32bit - index number addr.
 - ONLY WORK WITH -> Ring 0 HighPreveliged Instructions
   - RDMSR : READ_MSR ->  Read  data of [index in ECX] into EDX:EAX 
   - WRMSR : WRITE_MSR => Write data into [index in ECX] from EDX:EAX 
 
> 























































