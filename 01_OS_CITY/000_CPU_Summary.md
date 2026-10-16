
# SUMMARY: LIST
---
## REF: [TrapCall](./501_TrapCall.md)
## REF: [Kernal Stack Pointer](./502_KernalStackPointer.md)

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

###### [TSS] is ❌ NOT ==HARDWARE REGISTER== inside the CPU chip 
###### Its an [array of bytes] created by the ==OS Kernel== and stored in [RAM]. 
###### Physical CPU [core] only contains a tiny register -> [TR (Task Register)]
###### [TR] holds => ==OS writes== [pointer pointing to TSS struct in RAM] ON [TR]
### Hence: [TR] can be anything => [RSP]-0,1,2,3 or [ISP]-0to7

```rust (RUST STYLE)
/// Previliged Checking.
fn Previliged_work(...)->Resualt<()>{
  if Privileged_Instruction {
    if CPL_or_EL(Ring 0 || EL1){ // G: KERNEL Mode
      OK()
    }else{ 
      CPU::hardware_block || process::no_execution ;
      eprintln("Exception");
    }
  } 
}

enum Mode{
  User
  Kernel
}

fn swithc_active_stack_pointer(target:Mode)->Resualt<()>{
  match target{
    Kernel => {
      ACTIVE_STACK_PTR = CPU::Csr::Read::CPL::Ring_0.as_ref();
      ACTIVE_STACK_PTR = CPU_HW::Cpsr::Write::EL::EL_1.as_ref();
    },
    User =>{
      ACTIVE_STACK_PTR = CPU::Csr::Read::CPL::Ring_3.as_ref();
      ACTIVE_STACK_PTR = CPU_HW::Cpsr::Write::EL::EL_0.as_ref();
    }
  }
}

fn TRAP::SYS_CALL{
  CPU_HW::Csr::Write::CPL::Ring_0; 
  CPU_HW::Cpsr::Write::EL::EL_1;

  swithc_active_stack_pointer(Kernel);
}

fn TRAP::SYS_RET{
  CPU_HW::Csr::Write::CPL::Ring_3; // x86_64 CPL upgrade
  CPU_HW::Cpsr::Write::EL::EL_0; // ARM64 EL upgarde

  swithc_active_stack_pointer(User);
}

fn sys_call_fork(){

  todo() // Define registors
  TRAP::SYS_CALL;

  todo(
    ...
    let x = Previliged_work();
    ...
  ) // copying of process logic

  TRAP::SYS_RET;
}

/// NOT A REAL CODE
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
  > - [ALL 7 ISTS]: Usage:
     🔴 IST1:(NMI Stack) -> Non Maksable Intrupts Safety -> Points Emergency Stack -> Used at Catastrophic HW Events.
     ⚫ IST2:(Double Fault Stack:to Stop Tripple Crash) -> if 
          if CPU Crash := Handeler => Handeling time Crash := (imposible to handle so) -> another EMERGENCY STACK to REBOOT.
     🟡 IST3:(Machine Check Stack) -> Severlal (Cpu & Memory) HW errors.
     🟢 IST4:(Debug Stack) -> Used for Kernal Debuging -> HW breakpoints settings without interfering Normal Codes.
     🔵 IST5:(MCE Safe Recovery Stack) -> NEW Kernals: Specific MEM-Errors Safe Recovery.
     🟣 IST6 & IST7 => RESERVED FOR FUTURE extensions. 

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
    - RIP [Reg Instruction Pointer] ==Program Counter== => Addr of Very Next Machine Code.
    - RSP [Reg Stack Pointer] => (Top of Current App / Kernel Stack) -> in RAM.
    - CS [Code Segment] => Tracks [CPR] at lowest 2 bites
    - TR [Taks Reg] => Point to TSS (Process OR Kernel) in RAM.

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

  + 





























