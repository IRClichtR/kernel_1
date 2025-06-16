
# 🧠 Introduction to Assembly Language & Registers

## 📦 What Are Registers?

Registers are small, super-fast memory locations **inside the CPU**. They're used to temporarily hold data during execution.  
There are only a limited number of them, and their size depends on the processor (e.g. 16, 32, or 64 bits).

---

## 🧩 Types of Registers

1. **General-purpose registers:** `ax`, `bx`, `cx`, `dx`  
2. **Index and pointer registers:** `si`, `di`, `sp`, `bp`  
3. **Segment registers:** `cs`, `ds`, `es`, `ss`  
4. **Flags register:** Holds condition flags (e.g. zero, carry, overflow)  
5. **Instruction pointer (`ip`)**: Points to the current instruction being executed

---

## 🔍 Special Features of General-Purpose Registers

Registers like `ax`, `bx`, etc. can be split into high and low 8-bit parts:
- `ah` (high byte)
- `al` (low byte)

Changing either affects the full `ax` register.

---

## 📏 Extended Registers (32 and 64 bits)

With newer CPUs (starting with the 80386), registers were extended:
- `ax` → `eax` (32-bit)
- `eax` → `rax` (64-bit, in x86_64)

---

## 🔍 What Determines Available Registers?

1. **CPU architecture** (32 or 64-bit)
2. **Assembly mode** (x86 or x86_64)
3. OS (indirectly)

---

## 💡 Examples

### 🧱 In 32-bit Mode:
- Registers: `eax`, `ebx`, `ecx`, etc.
- System calls: `int 0x80`
- Ubuntu still supports this, but may need extra dependencies

### 🏢 In 64-bit Mode:
- Registers: `rax`, `rbx`, `rcx`, etc., and `r8`–`r15`
- System calls: `syscall`
- This is default on modern Ubuntu systems

---

## 📘 What is ABI?

**ABI (Application Binary Interface)** defines how programs interact at the binary level. It specifies:
- How arguments are passed
- Where return values go
- Which registers must be preserved
- Stack organization

---

## 🔁 What Are Calling Conventions?

Calling conventions are part of the ABI that define **how to call and return from functions**.

### Example: System V AMD64 (used in Linux x86_64)

| Argument | Register |
|----------|----------|
| 1st      | `rdi`    |
| 2nd      | `rsi`    |
| 3rd      | `rdx`    |
| 4th      | `rcx`    |
| 5th      | `r8`     |
| 6th      | `r9`     |

- Return values are in `rax`
- Caller must preserve: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`, `rax`
- Callee must preserve: `rbx`, `rbp`, `r12–r15`

---

## 🧠 Why Does This Matter?

- Misusing registers leads to bugs or crashes
- Essential when calling system or library functions

---

## ✅ System Call Example

```asm
mov rax, 1      ; syscall number for write
mov rdi, 1      ; stdout
mov rsi, msg    ; pointer to message
mov rdx, len    ; message length
syscall         ; invoke kernel
```

---

## 📚 Resources

- https://github.com/0xAX/asm/blob/master/content/asm_1.md
- http://sdz.tdct.org/sdz/en-profondeur-avec-l-assembleur.html
