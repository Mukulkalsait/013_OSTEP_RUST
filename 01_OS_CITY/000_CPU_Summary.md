
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
  - [IST] => Interrupt Stack Table (O to 7) -> OS use High Priority Intruptions.
    + use stack ***Prevention** total system failure*
    + Lives in TSS (ram) use TR Logical CPUs.
    + Emergency Handlers 
  -




















