
--- 
## 1. What is a "Trap" in the Simplest Way Possible?

  - A-> User Program in USER MODE => Sandboxed, Powerless, NO-HW access.
  - User program has Trigger -> Emergency Liver [🆘 SYS-CALL].
  - The movement SYS_CALL Triggered ->
   1. CPU freez => user program.
   2. CPU hardware flips [Privilege Ring (x86_64) | CPSR ->Exception Levels (ARM64)]  => **UserMode => KernelMode**
      - arm CPSR => [CurrentProgramStateRegistor]
      - + Previliged Level/ExecutionLevel (EL)-> 0=(user) | 1=(kernel) | 2=(Hypervisor)
      - + ALU:StateFlags -> (Z)ero, (C)arry, (N)egative, O(V)erflow.
   3. CPU jumps to a safe, pre-designated highway leading straight into the OS kernel.

## 1. The Privilege Ring / Execution Mode Register (The Current Privilege Level)
##### CPU contains a SET OF HARDWARE REGISTER => dictates -> Current Security State.

  * In x86_64: 
    -  CSR:[code segment reg] => Contian CPL [Current Privilege Level] at lowest 2 bites 
    -  16-bit CSR => small package
      - CPU reads last two binary digits (bits 0 and 1) to determine the CPL.
      - + 2 bits, the CPL can be a number from 0 to 3 (binary 00, 01, 10, or 11).

  * In ARM64: 
    - EL:[Exception Levels] => EL0,EL1,EL2.

| CPU | Name | User | Kernel | Other | 
| --- | --- | --- | --- | --- |
| x86_64 | Current Previliged Level | Ring 3 | Ring 0 | Ring 1/2 => IO Drivers |
|  | ==Lower 2 bytes or CSR== | ==total 16bit== | ==last2bit== | ==Val(0,1,2,3)== |
| ARM64 | Exception Levels | EL0 | EL1 | EL2 (Hypervisor) | 
|  | ==+ ALU-StateFlag== | ==Z,C,N,O== |  |[Zero,Carry,Negative,oVerdrive]  | 


  * What it does physically: 
    - IF CPU gets [Previliged Instruction] (like modifying memory tables or halting the CPU),
    - CPU CHECKS -> [CPL / EL] 
    - If bits -> "User Mode" (Ring 3/EL0),
       - ⛔ CPU hardware PHYSICAL-BLOCKS -> instruction & EXECUTION.
       - ⛔  throws an EXCEPTION.

--- 

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
