# PotatOS🥔

[toc]

## 写在前面

### 🥔什么是PotatOS

PotatOS是一个用 Rust 编写的基于 [2024 春夏季开源操作系统训练营 rCore 项目](https://github.com/bosswnx/2024s-rcore-bosswnx)的 RISC-V 架构的兼容 POSIX 协议的操作系统内核，实现了基本的进程管理、内存管理、文件系统和设备驱动。

### 开发环境

- 主机：x86_64 Ubuntu 22.04 LTS
- 工具链：Rust (nightly) + riscv64gc-unknown-elf-gcc
- 调试：QEMU + GDB + VSCode (rust-analyzer)

### 文件结构

```shell
.
├── bootloader
├── easy-fs
├── easy-fs-fuse
├── Makefile
├── os
├── qemu-7.0.0
├── rust-toolchain.toml
├── setenv.sh
└── user
```

- **bootloader**：Qemu 模拟器启动时的引导加载程序。它负责在系统启动时初始化硬件，并将控制权交给操作系统内核。
- **easy-fs**：easy file system，是本系统采用的简化文件系统，负责文件的存储和管理。
- **easy-fs-fuse**：efs filesystem in userspace，用于测试 efs，并且能够把内核开发的应用打包成一个 efs 格式的文件系统镜像，方便在模拟器中使用。
- **Makefile**：包含了一系列编译和运行的规则，通过`make`命令可以方便地编译和运行操作系统。
- **os**：内核文件夹，存放操作系统内核的源代码。
- **qemu-7.0.0**：Qemu 文件夹，包含了 Qemu 模拟器的相关文件。
- **rust-toolchain.toml**：rust 工具链描述文件，指定了使用的 Rust 版本和相关配置。
- **setenv.sh**：设置开发环境的脚本，运行该脚本可以配置必要的环境变量。
- **user**：用户空间文件夹，存放用户应用程序的源代码。



## 执行环境与平台

- **架构**：RISC-V RV64GC（特权模式：S-mode）

- **QEMU 配置**：

  ```bash
  qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios none \
    -kernel target/riscv64gc-unknown-none-elf/debug/potatos \
    -device virtio-blk-device,drive=disk0 \
    -drive file=fs.img,format=raw,id=disk0
  ```
  
  - `-machine virt`：指定使用 QEMU 的虚拟硬件模型。
  - `-nographic`：不使用图形界面，以文本模式运行。如果需要图形化界面，可以去掉该选项。
  - `-bios none`：不使用 BIOS。
  - `-kernel target/riscv64gc-unknown-none-elf/debug/potatos`：指定要加载的内核二进制文件。
  - `-device virtio-blk-device,drive=disk0`：添加一个 Virtio 块设备。
  - `-drive file=fs.img,format=raw,id=disk0`：指定使用的磁盘镜像文件。

### RustSbi

`SBI`是RISC-V定义的一组接口规范，用于操作系统与硬件间的沟通。`RustSbi`是连接该操作系统与底层硬件的桥梁，主要有以下功能：

- **硬件抽象**：将硬件访问抽象成接口，使得操作系统内核可以通过统一的接口访问不同的硬件设备，提高了代码的可移植性。
- **系统启动**：上电时 Sbi 负责初始化硬件，然后从存储中加载到操作系统。它会完成一些必要的硬件初始化工作，如设置内存映射、初始化中断控制器等，为操作系统的启动做好准备。
- **异常和中断处理**：当发生异常或中断时，操作系统把硬件控制权交给 Sbi，Sbi 在底层完成处理后交还控制权。Sbi 可以处理一些底层的异常和中断，如定时器中断、外部设备中断等，减轻了操作系统内核的负担。



## Boot

通过Qemu启动

```makefile
QEMU_ARGS := -machine virt \
			 -bios $(BOOTLOADER) \
			 -serial stdio \
			 $(GUI_OPTION) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
			 -drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			 -device virtio-blk-device,drive=x0 \
			 -device virtio-gpu-device \
			 -device virtio-keyboard-device \
			 -device virtio-mouse-device \
			 -device virtio-net-device,netdev=net0 \
			 -netdev user,id=net0,hostfwd=udp::6200-:2000,hostfwd=tcp::6201-:80
```

将编译好的kernel二进制文件加载到对应的启动位置，这里是`0x80200000`



## 中断/特权级机制

RISC-V为了给操作系统一个稳定的运行环境，不会被应用所干扰，设计了一个特权级机制。这里我们主要是实现这一部分的功能。

### 切换

1. **用户程序执行触发 Trap**：当用户程序执行某些特殊指令或发生异常时，会触发 Trap 机制，将控制权转移到操作系统内核。
2. **硬件自动保存 `sepc`、`sstatus` 等寄存器到内核空间的 `TrapContext`**：`sepc` 保存了 Trap 发生时的程序计数器，`sstatus` 保存了当前的状态信息。硬件会自动将这些寄存器的值保存到内核空间的 `TrapContext` 中，以便后续恢复。
3. **内核读取 `TrapContext`，执行 `trap_handler` 处理逻辑（如系统调用服务）**：内核从 `TrapContext` 中读取相关信息，根据不同的 Trap 原因执行相应的处理逻辑，如处理系统调用、异常处理等。
4. **处理完毕后，从 `TrapContext` 恢复用户寄存器（`x`、`sstatus`），通过 `sret` 指令跳回 `sepc` 继续执行用户程序**：处理完 Trap 后，内核将用户寄存器的值从 `TrapContext` 中恢复，并通过 `sret` 指令将控制权交还给用户程序，继续执行后续的指令。

#### 用户栈/内核栈

区分两个栈是为了应用程序不会通过栈信息读取到内核的控制流，从而避免了一定的安全隐患。

但在PotatOS里，用户栈被简单地抽象成了**应用栈**

内核栈与进程绑定，抽象成了进程独立且隔离的**进程栈**

#### 上下文切换/恢复

```rust
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}
```

- `x`：保存32个通用寄存器的值，暂存用户空间的寄存器状态，以便陷阱处理完毕后恢复执行
- `sstatus`：保存超级用户状态寄存器(Sstatus)的值，记录当前特权级，中断使能，状态标志，用于状态恢复
- `sepc`：保存超级用户异常程序计数器(SEPC)的值，记录trap发生时的地址
- `kernel_satp` ：内核地址空间的 token ，即内核页表的起始物理地址
- `kernel_sp` ：当前应用在内核地址空间中的内核栈栈顶的虚拟地址
- `trap_handler` ：内核中 trap handler 入口点的虚拟地址

```assembly
.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .globl __alltraps_k
    .globl __restore_k
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret

    .align 2
```

- `__alltraps`：保存 trap 上下文至内核栈。在发生 Trap 时，该函数会将用户空间的寄存器状态保存到内核栈的 `TrapContext` 中，并切换到内核空间。
- `__restore`：恢复 trap 上下文。在处理完 Trap 后，该函数会从 `TrapContext` 中恢复用户寄存器的值，并切换回用户空间，继续执行用户程序。

```rust
pub fn trap_return() -> ! {
    current_task().unwrap().inner_exclusive_access(file!(), line!()).user_time_start();

    disable_supervisor_interrupt();
    set_user_trap_entry();
    let trap_cx_user_va = current_trap_cx_user_va();
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    //println!("before return");
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_user_va,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}
```

在trap恢复时调用上面的函数回到正确的栈空间内。

#### trap处理

当发生 Trap 时，操作系统会根据 Trap 的原因执行相应的处理逻辑。常见的 Trap 原因包括系统调用、异常和中断等。在 `trap_handler` 函数中，会根据不同的 Trap 原因进行分类处理，如处理系统调用时会根据系统调用号执行相应的服务函数，处理异常时会进行错误处理和恢复，处理中断时会调用相应的中断处理函数。

```rust
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    // println!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            current_task().unwrap().inner_exclusive_access(file!(), line!()).user_time_end();
            
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;

            enable_supervisor_interrupt();

            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            /*
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            */
            current_add_signal(SignalFlags::SIGSEGV);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            current_add_signal(SignalFlags::SIGILL);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            check_timer();
            suspend_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::board::irq_handler();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // check signals
    if let Some((errno, msg)) = check_signals_of_current() {
        println!("[kernel] {}", msg);
        exit_current_and_run_next(errno);
    }
    trap_return();
}
```

这里通过`match`语句实现中断向量查找-匹配-处理。

### 跳板(Trampoline)

在传统的OS中，一个应用的用户和内核态通常分配在同一个地址空间的高位和低位，这样可以方便地使用栈寄存器进行跳转。但是，这样做会有内核工作流泄露的风险。所以需要一个跳板来保存工作流调用访问链条。

<img src="C:\Users\Kid_A\AppData\Roaming\Typora\typora-user-images\image-20250530132531781.png" alt="image-20250530132531781" style="zoom: 50%;" />

```rust
// os/src/mm/memory_set.rs
fn map_trampoline(&mut self) {
    self.page_table.map(
        VirtAddr::from(TRAMPOLINE).into(),
        PhysAddr::from(strampoline as usize).into(),
        PTEFlags::R | PTEFlags::X,
    );
}
```

这里把虚拟地址的高位都固定映射到`trampoline`。

### 系统调用

在**RISC-V**里，系统调用主要由`ecall`实现。当用户程序需要执行特权操作，如文件读写、进程创建、内存管理等时，无法直接访问硬件资源，必须通过 `ecall` 指令请求操作系统内核提供服务。

```rust
// usr/src/syscall.rs
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

// os/src/trap/mod.rs
Trap::Exception(Exception::UserEnvCall) => {
    current_task().unwrap().inner_exclusive_access(file!(), line!()).user_time_end();    
    // jump to next instruction anyway
    let mut cx = current_trap_cx();
    cx.sepc += 4;
    enable_supervisor_interrupt();
    // get system call return value
    let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
    // cx is changed during sys_exec, so we have to call it again
    cx = current_trap_cx();
    cx.x[10] = result as usize;
}

// os/src/syscall/mod.rs
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_FSTAT => sys_fstat(args[0], args[1]),
        SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        // ...
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
```

`syscall`通过`ecall`发起`UserEnvCall`，传递函数名(枚举)和参数，找到并执行对应的系统调用函数。

> ecall
>
> 1. 系统调用接口
>
> `ecall` 指令的主要用途是实现**系统调用**（System Call）。当用户程序需要执行特权操作（如文件读写、进程创建、内存管理等）时，无法直接访问硬件资源，必须通过 `ecall` 指令请求操作系统内核提供服务。
>
> 2. 特权级切换
>
> - **用户模式（U-mode）**：用户程序运行在此模式，权限受限，无法直接访问硬件或执行特权指令。
> - **监管者模式（S-mode）**：操作系统内核运行在此模式，拥有完整的硬件访问权限。
> - **`ecall` 的作用**：将处理器从 U-mode 切换到 S-mode，并跳转到内核预先设置的**陷阱处理程序**（Trap Handler）。

## 地址空间管理

OS需要提供虚拟地址，需要进行物理地址和虚拟地址的转换，需要动态分配地址空间。

### 内存管理

在PotatOS中，内存主要由`page, page table, frame`管理。

- **page**：虚拟内存的单位，通常为 4KiB。操作系统将虚拟地址空间划分为多个页面，方便进行内存管理和保护。
- **frame**：物理内存的单位，也通常为 4KiB。物理内存被划分为多个帧，用于存储页面的数据。
- **page_table**：虚拟地址到物理地址的转换表。通过页表，操作系统可以将虚拟地址映射到对应的物理地址，实现虚拟内存和物理内存的分离。

### 内存管理者模型

我们通过固定分配的**HEAP_ALLOCATOR**作为内核的堆存储空间，**FRAME_ALLOCATOR**作为动态分配的栈空间。他们分别从地址空间的开始/结尾分配内存。

##### HEAP-ALLOCATOR

简单地使用一段数组来分配数据

```rust
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

```

- `HEAP_ALLOCATOR`：全局堆分配器，用于管理内核的堆内存。
- `handle_alloc_error`：堆分配错误处理函数，当堆分配失败时会触发该函数。
- `HEAP_SPACE`：堆内存空间，是一个固定大小的数组。
- `init_heap`：初始化堆分配器，将堆内存空间的起始地址和大小传递给堆分配器。

##### FRAME_ALLOCATOR

使用简单的**双指针标记**，顺序遍历来分配物理页帧。设置了全局静态`FRAME_ALLOCATOR`来包装并分配。

**ekernel**表示数据段的结尾。尽管形式上是这样的，我还是倾向于栈由高位向低位分配。

```rust
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn alloc_more(&mut self, pages: usize) -> Option<Vec<PhysPageNum>> {
        if self.current + pages >= self.end {
            None
        } else {
            self.current += pages;
            let arr: Vec<usize> = (1..pages + 1).collect();
            let v = arr.iter().map(|x| (self.current - x).into()).collect();
            Some(v)
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPIntrFreeCell<FrameAllocatorImpl> =
        unsafe { UPIntrFreeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access(file!(), line!()).init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access(file!(), line!())
        .alloc()
        .map(FrameTracker::new)
}

pub fn frame_alloc_more(num: usize) -> Option<Vec<FrameTracker>> {
    FRAME_ALLOCATOR
        .exclusive_access(file!(), line!())
        .alloc_more(num)
        .map(|x| x.iter().map(|&t| FrameTracker::new(t)).collect())
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access(file!(), line!()).dealloc(ppn);
}
```

- `StackFrameAllocator`：栈帧分配器，使用双指针标记和回收机制来管理物理页帧。
- `init`：初始化栈帧分配器，设置起始和结束地址。
- `alloc`：分配一个物理页帧。如果有回收的页帧，则优先使用；否则，从当前位置分配一个新的页帧。
- `alloc_more`：分配多个物理页帧。
- `dealloc`：释放一个物理页帧，并将其加入回收列表。
- `FRAME_ALLOCATOR`：全局帧分配器，使用 `lazy_static` 进行静态初始化。
- `init_frame_allocator`：初始化帧分配器，设置分配范围。
- `frame_alloc`：分配一个帧，并返回一个 `FrameTracker` 对象。
- `frame_alloc_more`：分配多个帧，并返回一个 `FrameTracker` 对象的向量。
- `frame_dealloc`：释放一个帧。

### 多级页表管理地址空间(SV39)

#### 概念

我们希望实现物理地址和虚拟地址的转换和分页管理地址空间，因此需要一个标准。基于SV39实现的地址符合以下要求。

**地址格式**

![../_images/sv39-va-pa.png](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/sv39-va-pa.png)

**页表格式**

![../_images/sv39-pte.png](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/sv39-pte.png)

可以发现我们划分了 4KiB 大小的 page 用于对齐。SV39 是 RISC-V 架构中的一种页表机制，它将 39 位的虚拟地址划分为三个 9 位的虚拟页号和 12 位的页内偏移，将 56 位的物理地址划分为物理页号和页内偏移。通过多级页表的方式，实现虚拟地址到物理地址的转换。

#### Address

按照标准实现了地址的包装，包括地址之间的转换，page内bits的读取等等

```rust
// os/src/mm/address.rs
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
/// PAGE_SIZE_BITS = 0x12(4KiB)
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

/// Definitions
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

/// Debugging
/// ...

/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
/// usize -> T: usize.into()

/// transformations of addresses
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}
impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

/// get page num by address
impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        if self.0 == 0 {
            PhysPageNum(0)
        } else {
            PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

impl PhysAddr {
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        pa.get_mut()
    }
}
```

- `PhysAddr`：物理地址结构体，包装了一个 `usize` 类型的物理地址。
- `VirtAddr`：虚拟地址结构体，包装了一个 `usize` 类型的虚拟地址。
- `PhysPageNum`：物理页号结构体，包装了一个 `usize` 类型的物理页号。
- `VirtPageNum`：虚拟页号结构体，包装了一个 `usize` 类型的虚拟页号。
- `From` 实现：提供了从 `usize` 类型到各种地址和页号类型的转换。
- `From`、`From`、`From`、`From` 实现：提供了从各种地址和页号类型到 `usize` 类型的转换。
- `VirtAddr` 和 `PhysAddr` 的方法：提供了获取页号、页内偏移、判断对齐等功能。
- `VirtPageNum` 的 `indexes` 方法：将虚拟页号拆分为三个 9 位的索引。
- `PhysAddr` 和 `PhysPageNum` 的方法：提供了获取引用、修改引用、获取页表项数组等功能。

#### PageTable

##### 概念

**PageTable**主要负责转换虚拟地址和物理地址。SV39有三级页表机制，需要简单的循环和标记位验证解决问题。以下为**xv6**页表变换示意图，同理。

![../_images/sv39-full.png](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/sv39-full.png)

在 SV39 模式中我们采用三级页表，即将 27 位的虚拟页号分为三个等长的部分，第 26-18 位为一级页索引 **VPN0** ，第 17-9 位为二级页索引 **VPN1** ，第 8-0 位为三级页索引 **VPN2** 。

我们也将页表分为一级页表（多级页表的根节点），二级页表，三级页表（多级页表的叶节点）。每个页表都用 9 位索引，因此有 29=512 个页表项，而每个页表项都是 8 字节，因此每个页表大小都为 512×8=4KiB 。正好是一个物理页的大小。我们可以把一个页表放到一个物理页中，并用一个物理页号来描述它。事实上，一级页表的每个页表项中的物理页号可描述一个二级页表；二级页表的每个页表项中的物理页号可描述一个三级页表；三级页表中的页表项内容则和我们刚才提到的页表项一样，其内容包含物理页号，即描述一个要映射到的物理页。

具体来说，假设我们有虚拟地址 (VPN0,VPN1,VPN2,offset) ：

- 我们首先会记录装载「当前所用的一级页表的物理页」的页号到 satp 寄存器中；
- 把 VPN0 作为偏移在一级页表的物理页中找到二级页表的物理页号；
- 把 VPN1 作为偏移在二级页表的物理页中找到三级页表的物理页号；
- 把 VPN2 作为偏移在三级页表的物理页中找到要访问位置的物理页号；
- 物理页号对应的物理页基址（即物理页号左移12位）加上 offset 就是虚拟地址对应的物理地址。

这样处理器通过这种多次转换，终于从虚拟页号找到了一级页表项，从而得出了物理页号和虚拟地址所对应的物理地址。刚才我们提到若页表项满足 R,W,X 都为 0，表明这个页表项指向下一级页表。在这里一级和二级页表项的 R,W,X 为 0 应该成立，因为它们指向了下一级页表。

```rust
// os/src/mm/page_table.rs
fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
    let idxs = vpn.indexes();
    let mut ppn = self.root_ppn;
    let mut result: Option<&mut PageTableEntry> = None;
    for (i, idx) in idxs.iter().enumerate() {
        let pte = &mut ppn.get_pte_array()[*idx];
        if i == 2 {
            result = Some(pte);
            break;
        }
        if !pte.is_valid() {
            let frame = frame_alloc().unwrap();
            *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
            self.frames.push(frame);
        }
        ppn = pte.ppn();
    }
    result
}
fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
    let idxs = vpn.indexes();
    let mut ppn = self.root_ppn;
    let mut result: Option<&mut PageTableEntry> = None;
    for (i, idx) in idxs.iter().enumerate() {
        let pte = &mut ppn.get_pte_array()[*idx];
        if i == 2 {
            result = Some(pte);
            break;
        }
        if !pte.is_valid() {
            return None;
        }
        ppn = pte.ppn();
    }
    result
}
```

- `find_pte_create`：查找并创建页表项。如果页表项不存在，则分配一个新的物理页帧，并创建页表项。
- `find_pte`：查找页表项。如果页表项不存在，则返回 `None`。

##### 映射

每个进程有自己的虚拟地址，映射到不同的物理地址中。需要用页表进行映射/取消映射和物理地址/虚拟地址的转换。

这里我们采用最简单的恒等映射，即`ppn=vpn`的方式映射。

同时，为了区分每个进程的页表，使用`Token`分辨页表。

```rust
// os/src/mm/page_table.rs
#[allow(unused)]
pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
    let pte = self.find_pte_create(vpn).unwrap();
    assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
    *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
}
#[allow(unused)]
pub fn unmap(&mut self, vpn: VirtPageNum) {
    let pte = self.find_pte(vpn).unwrap();
    assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
    *pte = PageTableEntry::empty();
}
pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    self.find_pte(vpn).map(|pte| *pte)
}
/// ppn + offset
pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
    self.find_pte(va.clone().floor()).map(|pte| {
        let aligned_pa: PhysAddr = pte.ppn().into();
        let offset = va.page_offset();
        let aligned_pa_usize: usize = aligned_pa.into();
        (aligned_pa_usize + offset).into()
    })
}
pub fn token(&self) -> usize {
    8usize << 60 | self.root_ppn.0
}
```

- `map`：将虚拟页号映射到物理页号，并设置相应的标志位。
- `unmap`：取消虚拟页号的映射。
- `translate`：将虚拟页号转换为页表项。
- `translate_va`：将虚拟地址转换为物理地址。
- `token`：生成页表的 token，用于区分不同的页表。

### 地址空间管理

#### 逻辑段

前面我们实现了页帧管理和映射机制，但是这太零散了。我们还需要一层抽象来组织页帧。所以逻辑段就是这样的抽象。它组织了一段**连续且可用**的虚拟地址。

```rust
// os/src/mm/memory_set.rs
pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}
// ...
pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
    let ppn: PhysPageNum;
    match self.map_type {
        MapType::Identical => {
            ppn = PhysPageNum(vpn.0);
        }
        MapType::Framed => {
            let frame = frame_alloc().unwrap();
            ppn = frame.ppn;
            self.data_frames.insert(vpn, frame);
        }
        MapType::Linear(pn_offset) => {
            // check for sv39
            assert!(vpn.0 < (1usize << 27));
            ppn = PhysPageNum((vpn.0 as isize + pn_offset) as usize);
        }
    }
    let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
    page_table.map(vpn, ppn, pte_flags);
}
pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
    if self.map_type == MapType::Framed {
        self.data_frames.remove(&vpn);
    }
    page_table.unmap(vpn);
}
```

- `MapArea`：逻辑段结构体，包含虚拟页号范围、数据帧映射、映射类型和映射权限。
- `vpn_range`：一段左右闭合的虚拟地址范围。
- `data_frames`：物理页帧和虚拟地址的映射关系，使用 `BTreeMap` 存储。
- `map_type`：存在几种映射方式，包括直接映射、线性映射和新建页表随机映射。
  - **直接映射**：`ppn = vpn`，即虚拟页号和物理页号相同。
  - **线性映射**：通过一个偏移量来计算物理页号。
  - **新建页表随机映射**：分配一个新的物理页帧，并进行映射。
- `map_perm`：是否允许映射，指定了映射的权限，如读、写、执行等。
- `map_one`：按顺序取得 frame 进行映射。根据映射类型选择合适的物理页号，并调用页表的 `map` 方法进行映射。
- `unmap_one`：删除已映射的 vpn。如果是 `Framed` 映射类型，还需要从 `data_frames` 中移除相应的映射关系，并调用页表的 `unmap` 方法取消映射。

#### 地址空间

相当于为进程组织了一系列逻辑段，给每个进程分配了一个`PageTable`和`MapArea`来绑定进程的地址空间。这样每个进程就有了独立的地址空间。非常好的想法。

```rust
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}
```

基本就是提供了逻辑段的调用接口。

##### 内核地址空间

内核地址空间是操作系统内核所使用的地址空间，通常包含内核代码、数据、栈等。内核地址空间是所有进程共享的，它提供了对系统资源的直接访问，如设备驱动、中断处理等。在 PotatOS 中，内核地址空间的映射和管理是通过 `PageTable` 和 `MapArea` 来实现的。内核地址空间的映射通常是静态的，在系统启动时就已经完成。

<div style="float:left;border:solid 1px 000;margin:2px;"><img src="https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/kernel-as-high.png"  width="300" height="360" ></div>
<div style="float:left;border:solid 1px 000;margin:2px;"><img src="https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/kernel-as-low.png" width="300" height="360" ></div>
















内核地址的分布，高位是应用的内核栈，低位是内核地址空间的逻辑段，按照顺序插入。

```rust
pub fn new_kernel() -> Self {
    let mut memory_set = Self::new_bare();
    // map trampoline
    memory_set.map_trampoline();
    // map kernel sections
    // println!("mapping .text section");
    memory_set.push(
        MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::X,
        ),
        None,
    );
    // println!("mapping .rodata section");
    memory_set.push(
        MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
        ),
        None,
    );
    // println!("mapping .data section");
    memory_set.push(
        MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    // println!("mapping .bss section");
    memory_set.push(
        MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    // println!("mapping physical memory");
    memory_set.push(
        MapArea::new(
            (ekernel as usize).into(),
            MEMORY_END.into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    //println!("mapping memory-mapped registers");
    for pair in MMIO {
        memory_set.push(
            MapArea::new(
                (*pair).0.into(),
                ((*pair).0 + (*pair).1).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
    }
    memory_set
}
```



##### 应用地址空间

应用地址空间是每个进程独立拥有的地址空间，用于存储进程的代码、数据、栈等。每个进程的应用地址空间是相互隔离的，一个进程无法直接访问另一个进程的地址空间，从而保证了系统的安全性和稳定性。在 PotatOS 中，应用地址空间的映射和管理也是通过 `PageTable` 和 `MapArea` 来实现的。当创建一个新的进程时，会为其分配一个独立的页表和逻辑段，用于管理其应用地址空间。

![../_images/app-as-full.png](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/app-as-full.png)

实现了用户空间的统一化之后，我们可以把应用链接到同一个虚拟地址中

```assembly
OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x10000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
```



## 进程

进程可以简单理解为**操作系统对程序进行一次执行的过程**

在**rCore**中，进程被分离成了`task`和`process`。process主要指代进程，task主要指代具体执行的任务，可以理解为线程。

### 任务模型(Task)

任务是操作系统中最小的执行单位，它可以是一个线程或者一个进程中的一部分。任务通常具有自己的执行上下文，包括寄存器状态、栈指针等。在 PotatOS 中，任务的调度和管理是操作系统的重要功能之一，它负责决定哪个任务在何时执行，以提高系统的并发性能和资源利用率。

```rust
pub struct TaskControlBlock {
    pub pid: usize,
    pub ppid: usize,
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: UPIntrFreeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,

    pub user_time: usize,
    pub kernel_time: usize,
    pub time_created: usize,    
    pub first_time: usize,
    pub stop_watch: usize,
}
```

- `TCB`：任务的控制块，包含任务基本信息
  - `pid`：任务所属进程id
  - `ppid`：同理，父进程id
- `TCBInner`：任务内部信息，包括很多信息，可以通过互斥访问得到
  - `res`：返回状态
  - `cx`：任务的上下文，主要包括返回地址，栈顶位置和被调用者寄存器
  - `time`：任务的各种时间记录

**重要方法**：

主要是各种时间状态的设置，比如在trap时开启计时，下一个trap时停止当计时；

以及任务的调度。会在下面统一讲述。

### 进程模型(Process)

主要由`PCB`构成

```rust
pub struct ProcessControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: UPIntrFreeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub is_zombie: bool,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub signals: SignalFlags,
    pub tasks: Vec<Option<Arc<TaskControlBlock>>>,
    pub task_res_allocator: RecycleAllocator,
    pub mutex_list: Vec<Option<Arc<dyn Mutex>>>,
    pub semaphore_list: Vec<Option<Arc<Semaphore>>>,
    pub condvar_list: Vec<Option<Arc<Condvar>>>,

    pub cwd: Arc<Inode>,
}
```

- `ProcessControlBlock`：进程控制块结构体，包含进程的基本信息和可变部分。
  - `pid`：进程标识符，用于唯一标识一个进程。
  - `inner`：可变部分，使用 `UPIntrFreeCell` 进行封装，确保线程安全。
- `ProcessControlBlockInner`：进程控制块的可变部分，包含了进程的详细信息。
  - `is_zombie`：表示进程是否为僵尸进程。僵尸进程是指已经结束但尚未被父进程回收的进程。
  - `memory_set`：进程的地址空间，包含了进程的页表和逻辑段。
  - `parent`：父进程的弱引用，用于表示进程之间的父子关系。
  - `children`：子进程的强引用列表，存储了该进程的所有子进程。
  - `exit_code`：进程的退出码，用于表示进程的退出状态。
  - `fd_table`：文件描述符表，存储了进程打开的文件的引用。
  - `signals`：信号标志，用于处理进程接收到的信号。
  - `tasks`：任务列表，存储了进程中的所有任务。
  - `task_res_allocator`：任务资源分配器，用于分配和回收任务的资源。
  - `mutex_list`：互斥锁列表，用于实现进程内的互斥访问。
  - `semaphore_list`：信号量列表，用于实现进程间的同步和互斥。
  - `condvar_list`：条件变量列表，用于实现进程间的同步和通信。
  - `cwd`：当前工作目录的索引节点，用于指定进程的当前工作目录。

**重要方法**：

- `fork`：除了初始进程外，通用的进程产生方法。除了 pid 外，child 继承 parent 大部分信息。`fork` 系统调用会创建一个新的进程，该进程是调用进程的副本，拥有相同的代码、数据和文件描述符等。新进程的 pid 是唯一的，与父进程不同。
- `exec`：将当前进程的地址空间清空并加载一个特定的可执行文件，返回用户态后开始它的执行。`exec` 系统调用会用新的可执行文件替换当前进程的地址空间，从而使进程执行新的程序。
- `waitpid`：当前进程等待一个子进程变为僵尸进程，回收其全部资源并收集其返回值。`waitpid` 系统调用会阻塞当前进程，直到指定的子进程结束并变为僵尸进程，然后回收子进程的资源，并获取其退出码。

#### initproc

初始进程，所有进程都从它 fork 出来。这里的实现是直接 fork 出一个**user_shell**进程。`initproc` 是操作系统启动后的第一个进程，它负责创建其他进程和初始化系统环境。在 PotatOS 中，`initproc` 会 fork 出一个 `user_shell` 进程，用于提供用户交互界面。

#### user_shell

一个简单的 shell 程序，负责执行内部命令和外部命令。

- **内部命令**：关系到当前主进程，需要改变其状态的命令。比如`pwd`，`chdir`等等。内部命令通常是由 shell 本身实现的，不需要创建新的进程来执行。
- **外部命令**：执行系统外部的程序，简单地 fork 后 exec。外部命令是指需要执行系统中其他可执行文件的命令，shell 会创建一个新的进程来执行这些命令。



### 进程执行

#### 应用的链接与加载

在`Make`启动 Qemu 时，user/src/bin/ 内的程序会通过**efs-fuse**被预加载到 fs.img 中。详见下文。如果不使用 efs-fuse，则需要使用**link_app.S**把编译后的文件一个个加载到内存的地址中。这种硬编码的形式令人难受。

`efs-fuse` 是一个用户空间的文件系统，它可以将 user/src/bin/ 目录下的所有程序二进制格式打包，并加载到预先准备好的 fs.img 中。这样，在启动 Qemu 时，操作系统可以直接从 fs.img 中读取这些程序，并加载到内存中执行。而使用 `link_app.S` 则需要手动将编译后的文件加载到内存的指定地址，这种方式比较繁琐，而且容易出错。

### 任务调度

进程的调度基于**任务调度**。任务调度是操作系统的核心功能之一，它负责决定哪个任务在何时执行，以提高系统的并发性能和资源利用率。在PotatOS中，任务调度使用简单的RR时间片方法。

#### 调度方法

```rust
// os/src/task/manager.rs
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPIntrFreeCell<TaskManager> =
        unsafe { UPIntrFreeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPIntrFreeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPIntrFreeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access(file!(), line!()).add(task);
}

pub fn wakeup_task(task: Arc<TaskControlBlock>) {
    let mut task_inner = task.inner_exclusive_access(file!(), line!());
    task_inner.task_status = TaskStatus::Ready;
    let task_info = TaskInfo {
        pid: task.get_pid(),
        ppid: task.get_ppid(), 
        status: task_inner.task_status,
        user_time: task_inner.user_time,
        kernel_time: task_inner.kernel_time,
        time_created: task_inner.time_created,
        first_time: task_inner.first_time,
    };
    
    drop(task_inner);
    add_task(task);
    
    write_proc(task_info);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access(file!(), line!()).fetch()
}
```

任务的调度由简单的**FIFO队列**实现。主要包括一下几个方法：

- `add_task`：直接往队列里加入task
- `wakeup_task`：把一个新的task唤醒，然后插入到proces队列里
- `fetch_task`：取得一个task

#### RR

在内核里，我们设置了一个timer类，用于时间片轮转方法。

```rust
// os/src/timer.rs
pub fn get_time() -> usize {
    time::read()
}
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}
/// set time slice for current task
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn add_timer(expire_ms: usize, task: Arc<TaskControlBlock>) {
    let mut timers = TIMERS.exclusive_access(file!(), line!());
    timers.push(TimerCondVar { expire_ms, task });
}

pub fn check_timer() {
    let current_ms = get_time_ms();
    TIMERS.exclusive_session(|timers| {
        while let Some(timer) = timers.peek() {
            if timer.expire_ms <= current_ms {
                wakeup_task(Arc::clone(&timer.task));
                timers.pop();
            } else {
                break;
            }
        }
    });
}
```



### 进程调度

### 并发控制



## 文件系统

本系统模仿使用了**easyfs**，一个简化版本的文件系统。为了降低耦合性，整体可以分为五层：

1. **磁盘块接口层**：抽象封装磁盘块，实现对外接口。该层提供了对磁盘块的基本读写操作，是文件系统与磁盘设备之间的接口。
2. **块缓存层**：实现块缓存功能，提供对外读写接口。为了提高文件系统的读写性能，该层引入了块缓存机制，将经常访问的磁盘块缓存到内存中，减少了磁盘的读写次数。
3. **磁盘块数据结构层**：实现 superblock，datablock，inode 等等。该层定义了文件系统的基本数据结构，如超级块、数据块、索引节点等，用于管理文件和目录的存储。
4. **磁盘块管理层**：easyfs 的主要部分，进行块的管理。该层负责磁盘块的分配和回收，确保文件系统能够高效地利用磁盘空间。
5. **索引节点层**：inode 实现文件读写管理功能，封装后可以提供对外接口读写。该层通过索引节点来管理文件和目录的读写操作，为上层的操作系统提供了统一的文件访问接口。

通过 VFS 和 File trait 封装整个 easyfs 的 inode，为上层的 OS 提供接口。最后通过 VirtIO 模拟块设备驱动，搭载到 qemu 模拟器上面。

### Block

#### 块设备接口层

```rust
pub trait BlockDevice: Send + Sync + Any {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
    fn handle_irq(&self);
}
```

实现了读写块和处理中断的功能。该接口定义了块设备的基本操作，包括读取磁盘块、写入磁盘块和处理中断。任何实现了该接口的类型都可以作为块设备使用。

#### 块缓存层

为了应对频繁的读写采用缓存加速。

**块缓存**

```rust
pub struct BlockCache {
    cache: Vec<u8>,
    block_id: usize,
    block_device: Arc<dyn BlockDevice>,
    modified: bool,
}
```

- `cache`：缓存字节记录，用于存储磁盘块的内容。
- `block_id`，`block_device`：块设备以及块地址，用于标识缓存的磁盘块。
- `modified`：脏标记，采用懒更新方式刷入磁盘。如果该标记为 `true`，表示缓存中的数据已经被修改，需要在适当的时候将其写回磁盘。

提供了基本的读写和同步刷盘方法。可通过`get_block_cache`取得，通过传递一个闭包实现对应的方法和访问功能。块缓存的读写操作会先在缓存中查找，如果缓存中存在所需的数据，则直接从缓存中读取；如果缓存中不存在，则从磁盘中读取并更新缓存。同步刷盘方法会将缓存中被修改的数据写回磁盘。

**块缓存管理**

```rust
const BLOCK_CACHE_SIZE: usize = 16;
pub struct BlockCacheManager {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}
```

一个简单的缓存管理结构，实现了 LRU出入队和读写、同步方法。该管理器使用一个双端队列来维护缓存块的访问顺序，最近访问的缓存块会被移动到队列的头部，当缓存满时，会将队列尾部的缓存块淘汰。

### EasyFS

#### 概述

<img src="C:\Users\Kid_A\AppData\Roaming\Typora\typora-user-images\image-20250328175205292.png" alt="image-20250328175205292" style="zoom:50%;" />

基本结构如图所示

#### 超级块

```rust
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}
```

- `magic`：文件系统检查的魔数，用于标识文件系统的类型。
- `total_blocks`：文件系统的总块数，即磁盘上可用的总块数。
- `****_blocks`：给出了各个区域的块数，包括 inode 位图块数、inode 区域块数、数据位图块数和数据区域块数。

超级块实现了`debug`方法，即检查各部分块数，以及文件系统的检查，判定是否为EFS

#### Bitmap

```rust
const BLOCK_BITS: usize = BLOCK_SZ * 8;

pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}
```

- `start_block_id`：当前 bitmap 起始位置，即位图所管理的第一个磁盘块的编号。
- `blocks`：bitmap 管理了多少块，即位图所管理的磁盘块的数量。

bitmap 实现了基本的分配功能，包括分配和释放块时位图的改变。这里通过简单的线性遍历进行空块判断。当需要分配一个磁盘块时，位图会从起始位置开始线性遍历，找到第一个未被使用的块，并将其标记为已使用；当需要释放一个磁盘块时，位图会将该块标记为未使用。

#### DiskInode

```rust
const INODE_DIRECT_COUNT: usize = 27;
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
const INODE_INDIRECT3_COUNT: usize = INODE_INDIRECT2_COUNT * INODE_INDIRECT1_COUNT;
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;

pub enum DiskInodeType {
    File,
    Directory,
}

pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    pub indirect3: u32,
    type_: DiskInodeType,
    pub nlink: u32,
}
```

- `size`：当前占据的块大小，即文件或目录所占用的磁盘块数。
- `direct...`：直接索引和三级间接索引，用于管理文件或目录的数据块。直接索引可以直接指向数据块，而间接索引则通过索引块来指向数据块。
- `type_`：索引节点类型，分为文件和目录两种类型。
- `nlink`：硬链接数量，即指向该索引节点的硬链接的数量。

**DiskInode**是磁盘上文件（File or Directory）存储的基本形式。通过直接+间接索引的形式管理文件数据。在块获取，空间管理方面根据索引实现。有如下重要方法：

- `is_file` 和 `is_dir`：类型判断
- `increase_size`和`build_tree`：DiskInode空间增长和辅助函数。主要是多级索引的递归分配
- `clear_size`和`collect_tree_blocks`：释放DiskInode的方法和对应的多级索引递归释放
- `read_at`和`write_at`和`get_block_id`：读写方法和多级索引获取块ID方法

#### DitEntry（DEntry）

```rust
pub struct DirEntry {
    name: [u8; NAME_LENGTH_LIMIT + 1],
    inode_number: u32,
}
```

- `name`：Inode Name，即文件或目录的名称。
- `inode_number`：inode 唯一标识，用于唯一标识一个索引节点。

DirEntry 是 Inode 在 DiskInode 中存储的基本单位。固定为 32B 便于管理。实现了简单的取值方法。主要职能是从 inode block 指向 data block。通过目录项，可以将目录和文件关联起来，实现文件系统的目录结构。

#### EFS管理器

```rust
pub struct EasyFileSystem {
    pub block_device: Arc<dyn BlockDevice>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_area_start_block: u32,
    data_area_start_block: u32,
}
```

- `block_device`：管理块的块设备，用于读写磁盘块。
- `inode_bitmap`：管理 inode 分配的 bitmap，用于记录 inode 的使用情况。
- `data_bitmap`：管理数据块分配的 bitmap，用于记录数据块的使用情况。
- `inode_area_start_block`：inode 区域的起始块编号。
- `data_area_start_block`：数据区域的起始块编号。

EFS 作为整体的文件系统，对接的是磁盘管理和 VFS，为他们提供接口。实现方法：

- `create`：在块设备上创建 EFS，初始化文件系统的超级块、位图和索引节点等。
- `open`：打开块设备，读取文件系统的超级块和位图等信息，验证文件系统的完整性。
- `root_inode`：获取根节点，返回文件系统的根索引节点。
- `alloc_data`和`dealloc_data`：对接 bitmap 的接口，用于分配和释放数据块。

#### easy-fs-fuse

作用是能把 user/src/bin/ 内的所有程序二进制格式打包，加载到预先准备好的 fs.img 中。这样就不需要一个个链接进去了。`easy-fs-fuse` 是一个用户空间的文件系统，它可以将 user/src/bin/ 目录下的所有程序二进制格式打包，并加载到预先准备好的 fs.img 中。这样，在启动 Qemu 时，操作系统可以直接从 fs.img 中读取这些程序，并加载到内存中执行。

### VFS

#### 概述

连接 EFS 和 OS 内核的抽象层，为他们提供接口，并致力于实现透明化。VFS是一个虚拟文件系统层，它提供了一个统一的接口，使得操作系统内核可以通过相同的方式访问不同类型的文件系统。在 PotatOS 中，VFS 层将 EasyFS 和操作系统内核连接起来，为内核提供了统一的文件访问接口，使得内核可以方便地操作 EasyFS 文件系统。

#### Inode

VFS层主要就是为操作Inode提供接口

```rust
pub struct Inode {
    name: String,
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
    inode_id: u32,
}
```

- `name`：文件名，即文件或目录的名称。
- `block_id`，`block_offset`：Inode 在哪个块，块内偏移。可用于定位，通过块编号和块内偏移，可以准确地找到 Inode 在磁盘上的位置。
- `fs`，`block_device`：当前文件系统和块设备，用于访问文件系统和磁盘块。
- `inode_id`：inode 标识，用于唯一标识一个索引节点。

Inode 部分为 OS 实现了操作块的接口。可以简单把 Inode 看作文件 / 目录。

- `read_disk_inode`和`modify_disk_inode`：操作 DiskInode，用于读取和修改磁盘上的索引节点信息。
- `increase_size`：一些操作 DiskInode 的接口，用于扩展文件或目录的空间。
- `create_file`和`create_dir`：创建文件 / 目录，用于在文件系统中创建新的文件或目录。
- `linkat`和`unlinkat`：进行硬链接和作为删除辅助函数，用于创建和删除硬链接。
- 一些访问结构的函数，用于获取 Inode 的相关信息。

### File Trait & File System

#### File

- `write`和`read`：读写文件
- `stat`：文件状态，包括设备号、inode_id、文件类型、硬连接数和padding

#### File System

系统调用：

- `sys_write`，`sys_read`，`sys_open`，`sys_close`：读写，打开关闭文件

- `sys_getcwd`，`sys_fstat`：获取current work directory，获取fstat

- `sys_mkdir`，`sys_remove`，`remove_dir`：目录创建和删除的方法以及辅助函数

  

## IO设备管理

一个设备需要设备驱动进行管理。而一个设备驱动需要以下的功能：

1. **设备的扫描 / 发现**：检测系统中存在的设备，并识别其类型和特性。
2. **设备初始化**：对设备进行初始化，配置设备的寄存器和参数，使其处于可用状态。
3. **准备发送给设备的命令**：根据用户的请求，生成相应的命令，并准备发送给设备。
4. **通知设备**：将准备好的命令发送给设备，触发设备的操作。
5. **接受设备通知**：接收设备的响应和通知，处理设备的中断和状态变化。
6. **卸载设备的同时回收设备资源**：当设备不再使用时，卸载设备驱动，并回收设备所占用的资源。

这里主要实现了两种设备：

- **真实的物理设备**：如`URAT`，用于实现字符输入和输出。
- **虚拟设备**：如各种`Virtio`设备，用于模拟硬件设备的功能，提高系统的可移植性和兼容性。


**qemu特化**

本项目在 qemu 模拟器上运行，因此要基于 qemu 进行设备管理的特化。

在 qemu 里，IO 设备的交互以中断为主，轮询为辅。通过`PLIC 平台级中断控制器`进行。

**确定设备内存映射**

```rust
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc000000, 0x210000), // VIRT_PLIC in virt machine
    (0x10000000, 0x9000),  // VIRT_UART0 with GPU  in virt machine
];
```

`MMIO`确定了各个设备在qemu里的地址。通过这些地址操作系统可以直接读写设备寄存器进行交互

**设备初始化**

```rust
pub fn device_init() {
    use riscv::register::sie;
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let hart_id: usize = 0;
    let supervisor = IntrTargetPriority::Supervisor;
    let machine = IntrTargetPriority::Machine;
    plic.set_threshold(hart_id, supervisor, 0);
    plic.set_threshold(hart_id, machine, 1);
    //irq nums: 5 keyboard, 6 mouse, 8 block, 10 uart
    for intr_src_id in [5usize, 6, 8, 10] {
        plic.enable(hart_id, supervisor, intr_src_id);
        plic.set_priority(intr_src_id, 1);
    }
    unsafe {
        sie::set_sext();
    }
}
```

设备初始化，也是中断初始化，给每个设备启动中断功能。当中断发生时可以调用对应设备的中断响应函数。在设备初始化过程中，会对 PLIC（Platform-Level Interrupt Controller）进行配置，设置中断阈值和优先级，并使能相应的中断源。同时，会开启外部中断使能位，允许系统接收外部设备的中断请求。

#### 中断处理

```rust
pub fn irq_handler() {
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let intr_src_id = plic.claim(0, IntrTargetPriority::Supervisor);
    match intr_src_id {
        5 => KEYBOARD_DEVICE.handle_irq(),
        6 => MOUSE_DEVICE.handle_irq(),
        8 => BLOCK_DEVICE.handle_irq(),
        10 => UART.handle_irq(),
        _ => panic!("unsupported IRQ {}", intr_src_id),
    }
    plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
}
```

当发生中断时，操作系统会调用 `irq_handler` 函数进行处理。该函数会从 PLIC 中获取中断源的编号，并根据编号调用相应设备的中断处理函数。处理完中断后，会通知 PLIC 中断处理完成。

#### trap 响应

写入`scause`为外部中断，由qemu中断处理

```rust
// trap/mod.rs
// ...
Trap::Interrupt(Interrupt::SupervisorExternal) => {
    crate::board::irq_handler();
}
// ...
```

当发生超级用户外部中断时，操作系统会调用 `irq_handler` 函数进行处理。这样，操作系统可以及时响应外部设备的中断请求，处理设备的输入和输出。

### 串口驱动程序UART

**目的**

把字符输入到操作系统内核里。UART（Universal Asynchronous Receiver/Transmitter）是一种通用的异步收发传输器，用于实现字符的输入和输出。在 PotatOS 中，UART 驱动程序负责将用户输入的字符传输到操作系统内核，并将内核输出的字符发送到终端设备。

#### 初始化

**ns16550a**

```rust
pub struct NS16550aRaw {
    base_addr: usize,
}

impl NS16550aRaw {
    fn read_end(&mut self) -> &mut ReadWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut ReadWithoutDLAB) }
    }

    fn write_end(&mut self) -> &mut WriteWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut WriteWithoutDLAB) }
    }

    pub fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    pub fn init(&mut self) {
        let read_end = self.read_end();
        let mut mcr = MCR::empty();
        mcr |= MCR::DATA_TERMINAL_READY;
        mcr |= MCR::REQUEST_TO_SEND;
        mcr |= MCR::AUX_OUTPUT2;
        read_end.mcr.write(mcr);
        let ier = IER::RX_AVAILABLE;
        read_end.ier.write(ier);
    }

    pub fn read(&mut self) -> Option<u8> {
        let read_end = self.read_end();
        let lsr = read_end.lsr.read();
        if lsr.contains(LSR::DATA_AVAILABLE) {
            Some(read_end.rbr.read())
        } else {
            None
        }
    }

    pub fn write(&mut self, ch: u8) {
        let write_end = self.write_end();
        loop {
            if write_end.lsr.read().contains(LSR::THR_EMPTY) {
                write_end.thr.write(ch);
                break;
            }
        }
    }
}
```

通过`MMIO`进行通信，读写。内部是一堆寄存器。`NS16550aRaw` 是一个原始的 UART 设备结构体，它通过内存映射输入输出（MMIO）的方式与 UART 设备进行通信。在初始化过程中，会配置 UART 设备的寄存器，使能接收中断，并设置相应的控制位。`read` 方法用于从 UART 设备读取一个字符，`write` 方法用于向 UART 设备写入一个字符。

```rust
struct ReadWithoutDLAB {
    /// receiver buffer register
    pub rbr: ReadOnly<u8>,
    /// interrupt enable register
    pub ier: Volatile<IER>,
    /// interrupt identification register
    pub iir: ReadOnly<u8>,
    /// line control register
    pub lcr: Volatile<u8>,
    /// model control register
    pub mcr: Volatile<MCR>,
    /// line status register
    pub lsr: ReadOnly<LSR>,
    /// ignore MSR
    _padding1: ReadOnly<u8>,
    /// ignore SCR
    _padding2: ReadOnly<u8>,
}
```

`ReadWithoutDLAB` 结构体定义了 UART 设备的寄存器布局，包括接收缓冲区寄存器、中断使能寄存器、中断标识寄存器等。通过访问这些寄存器，可以实现对 UART 设备的控制和数据传输。

#### 中断处理

设备包装成`u8字节串`与内核进行中断交互

```rust
pub struct NS16550a<const BASE_ADDR: usize> {
    inner: UPIntrFreeCell<NS16550aInner>,
    condvar: Condvar,
}
```

通过信号量`condvar`实现内核部分的信号驱动 IO。当 UART 设备接收到字符时，会触发中断，中断处理函数会将接收到的字符存储到缓冲区中，并通过信号量通知内核有新的字符可用。内核可以通过等待信号量来获取新的字符。

```rust
fn handle_irq(&self) {
    let mut count = 0;
    self.inner.exclusive_session(|inner| {
        while let Some(ch) = inner.ns16550a.read() {
            count += 1;
            inner.read_buffer.push_back(ch);
        }
    });
    if count > 0 {
        self.condvar.signal();
    }
}
```

在中断处理函数中，会不断从 UART 设备读取字符，并将其存储到缓冲区中。如果读取到了字符，则通过信号量通知内核。这样，内核可以在有新的字符可用时被唤醒，提高了系统的响应性能。

#### 内核通信

UART作为输入设备设立成`stdin`

```rust
// stdio.rs

impl File for Stdin {
	// ...
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        assert_eq!(user_buf.len(), 1);
        //println!("before UART.read() in Stdin::read()");
        let ch = UART.read();
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    // ...
}
```

在进程创建中自动设置`stdin`，`stdout`，`stderr`。UART 设备作为标准输入设备（`stdin`），用户程序可以通过调用 `Stdin` 的 `read` 方法从 UART 设备读取字符。在进程创建时，会自动将 `stdin`、`stdout` 和 `stderr` 分别设置为 UART 设备，方便用户程序进行输入输出操作。

```rust
fd_table: vec![
    // 0 -> stdin
    Some(Arc::new(Stdin)),
    // 1 -> stdout
    Some(Arc::new(Stdout)),
    // 2 -> stderr
    Some(Arc::new(Stdout)),
],
```

此处错误输出自动导向终端

### Virtio Device

作为一个设备接口，允许虚拟机上运行的操作系统**通过访问virtio设备使用主机设备**。这里主要是利用它简单地实现虚拟设备。

### Virtio Block Device

我们希望通过操作系统内核对虚拟块设备进行简单的读写。

#### block-dev trait

```rust
pub trait BlockDevice: Send + Sync + Any {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
    fn handle_irq(&self);
}
```

块设备需要实现的`trait`，包括简单的读写和中断处理。在`文件系统`章节有提及。

#### virtio-block

```rust
pub struct VirtIOBlk<'a, H: Hal> {
    header: &'static mut VirtIOHeader,
    queue: VirtQueue<'a, H>,
    capacity: usize,
}

// drivers/block/virtio_blk.rs
pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<'static, VirtioHal>>,
    condvars: BTreeMap<u16, Condvar>,
}
```

包含了虚拟块设备的基本结构，包括virtio-driver和多通道通信。

`condvars`用于实现IO读写。事实上，一个`condvar`对应一个`virtio-queue`。virtio-queue是虚拟设备中通过**中断驱动的IO队列，支持轮询**。通过virtio-queue可以实现设备和驱动程序的各种数据传输工作。



#### 操作系统对接块设备初始化

```rust
impl VirtIOBlock {
    pub fn new() -> Self {
        let virtio_blk = unsafe {
            UPIntrFreeCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            )
        };
        let mut condvars = BTreeMap::new();
        let channels = virtio_blk.exclusive_access(file!(), line!()).virt_queue_size();
        for i in 0..channels {
            let condvar = Condvar::new();
            condvars.insert(i, condvar);
        }
        Self {
            virtio_blk,
            condvars,
        }
    }
}
```

初始化`virtio-block`和`channels`

```rust
// qemu.rs
pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;

// drivers/block/mod.rs
lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}
```

全局初始化块设备



#### 中断处理

同上，通过中断进行数据传输。

```rust
fn handle_irq(&self) {
    self.virtio_blk.exclusive_session(|blk| {
        while let Ok(token) = blk.pop_used() {
            self.condvars.get(&token).unwrap().signal();
        }
    });
}
```

从virtio-queue中取出已使用过的部分进行数据传输



### Virtio GPU Device

主要目的是为了**进程调度的图形化**而使用qemu虚拟现实设备。主要的功能是对显示设备内存进行数据读写。通过设置`显示屏尺寸`，`像素点位置`和`像素点颜色`可以实现基本的图形展示。像素点的放置由`cursor`辅助实现。

**简单的动画实现**

简单地考虑就是：首先程序绘制当前帧，然后屏幕刷新帧。

#### 数据结构

```rust
pub struct VirtIOGpu<'a, H: Hal> {
    header: &'static mut VirtIOHeader,
    rect: Rect,
    /// DMA area of frame buffer.
    frame_buffer_dma: Option<DMA<H>>,
    /// DMA area of cursor image buffer.
    cursor_buffer_dma: Option<DMA<H>>,
    /// Queue for sending control commands.
    control_queue: VirtQueue<'a, H>,
    /// Queue for sending cursor commands.
    cursor_queue: VirtQueue<'a, H>,
    /// Queue buffer DMA
    queue_buf_dma: DMA<H>,
    /// Send buffer for queue.
    queue_buf_send: &'a mut [u8],
    /// Recv buffer for queue.
    queue_buf_recv: &'a mut [u8],
}
```

为了提高系统执行效率，`像素内存`和`光标显示内存`由**DMA**管理并进行数据传输。

这里的DMA使用前文`内存管理`提到的函数方法直接访问物理/虚拟内存。

```rust
#[derive(Debug)]
pub struct DMA<H: Hal> {
    paddr: usize,
    pages: usize,
    _phantom: PhantomData<H>,
}

impl<H: Hal> DMA<H> {
    pub fn new(pages: usize) -> Result<Self> {
        let paddr = H::dma_alloc(pages);
        if paddr == 0 {
            return Err(Error::DmaError);
        }
        Ok(DMA {
            paddr,
            pages,
            _phantom: PhantomData::default(),
        })
    }

    pub fn paddr(&self) -> usize {
        self.paddr
    }

    pub fn vaddr(&self) -> usize {
        H::phys_to_virt(self.paddr)
    }

    /// Returns the physical page frame number.
    pub fn pfn(&self) -> u32 {
        (self.paddr >> 12) as u32
    }

    /// Convert to a buffer
    pub unsafe fn as_buf(&self) -> &'static mut [u8] {
        core::slice::from_raw_parts_mut(self.vaddr() as _, PAGE_SIZE * self.pages as usize)
    }
}

impl<H: Hal> Drop for DMA<H> {
    fn drop(&mut self) {
        let err = H::dma_dealloc(self.paddr as usize, self.pages as usize);
        assert_eq!(err, 0, "failed to deallocate DMA");
    }
}

/// The interface which a particular hardware implementation must implement.
pub trait Hal {
    /// Allocates the given number of contiguous physical pages of DMA memory for virtio use.
    fn dma_alloc(pages: usize) -> PhysAddr;
    /// Deallocates the given contiguous physical DMA memory pages.
    fn dma_dealloc(paddr: PhysAddr, pages: usize) -> i32;
    /// Converts a physical address used for virtio to a virtual address which the program can
    /// access.
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr;
    /// Converts a virtual address which the program can access to the corresponding physical
    /// address to use for virtio.
    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr;
}
```

`trait Hal`直接申请物理页表和物理地址，从而不使用中断直接访问地址空间。这样可以加速系统执行效率

```rust
impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access(file!(), line!())
            .append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        pa.0
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PhysAddr::from(pa);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        PageTable::from_token(kernel_token())
            .translate_va(VirtAddr::from(vaddr))
            .unwrap()
            .0
    }
}
```



#### 初始化虚拟GPU设备

这一步就是返回一个初始化后的GPU设备

```rust
pub fn new(header: &'static mut VirtIOHeader) -> Result<Self> {
        header.begin_init(|features| {
            let features = Features::from_bits_truncate(features);
            info!("Device features {:?}", features);
            let supported_features = Features::empty();
            (features & supported_features).bits()
        });

        // read configuration space
        let config = unsafe { &mut *(header.config_space() as *mut Config) };
        info!("Config: {:?}", config);

        let control_queue = VirtQueue::new(header, QUEUE_TRANSMIT, 2)?;
        let cursor_queue = VirtQueue::new(header, QUEUE_CURSOR, 2)?;

        let queue_buf_dma = DMA::new(2)?;
        let queue_buf_send = unsafe { &mut queue_buf_dma.as_buf()[..PAGE_SIZE] };
        let queue_buf_recv = unsafe { &mut queue_buf_dma.as_buf()[PAGE_SIZE..] };

        header.finish_init();

        Ok(VirtIOGpu {
            header,
            frame_buffer_dma: None,
            cursor_buffer_dma: None,
            rect: Rect::default(),
            control_queue,
            cursor_queue,
            queue_buf_dma,
            queue_buf_send,
            queue_buf_recv,
        })
    }
```

为了能够实现图形化，还需要建立**显示区域**，即**渲染帧**和**刷新帧**

```rust
pub fn setup_framebuffer(&mut self) -> Result<&mut [u8]> {
        // get display info
        let display_info = self.get_display_info()?;
        info!("=> {:?}", display_info);
        self.rect = display_info.rect;

        // create resource 2d
        self.resource_create_2d(
            RESOURCE_ID_FB,
            display_info.rect.width,
            display_info.rect.height,
        )?;

        // alloc continuous pages for the frame buffer
        let size = display_info.rect.width * display_info.rect.height * 4;
        let frame_buffer_dma = DMA::new(pages(size as usize))?;

        // resource_attach_backing
        self.resource_attach_backing(RESOURCE_ID_FB, frame_buffer_dma.paddr() as u64, size)?;

        // map frame buffer to screen
        self.set_scanout(display_info.rect, SCANOUT_ID, RESOURCE_ID_FB)?;

        let buf = unsafe { frame_buffer_dma.as_buf() };
        self.frame_buffer_dma = Some(frame_buffer_dma);
        Ok(buf)
    }
```

这一步设置了`显示设置`，即设备尺寸和分辨率。一个像素大小`4字节`，然后链接帧和屏幕。



#### 虚拟GPU设备IO操作

如上所言，GPU设备仅需要两步操作：

1. 渲染帧：把像素数据刷入显存内
2. 刷新帧：把新帧刷到屏幕上



#### 虚拟GPU驱动

```rust
pub trait GpuDevice: Send + Sync + Any {
    fn get_framebuffer(&self) -> &mut [u8];
    fn flush(&self);
}

lazy_static::lazy_static!(
    pub static ref GPU_DEVICE: Arc<dyn GpuDevice> = Arc::new(VirtIOGpuWrapper::new());
);

pub struct VirtIOGpuWrapper {
    gpu: UPIntrFreeCell<VirtIOGpu<'static, VirtioHal>>,
    fb: &'static [u8],
}

impl GpuDevice for VirtIOGpuWrapper {
    fn flush(&self) {
        self.gpu.exclusive_access(file!(), line!()).flush().unwrap();
    }
    fn get_framebuffer(&self) -> &mut [u8] {
        unsafe {
            let ptr = self.fb.as_ptr() as *const _ as *mut u8;
            core::slice::from_raw_parts_mut(ptr, self.fb.len())
        }
    }
}
```

虚拟GPU设备采用`DMA`，因此与操作系统的交互不需要进行地址变换，直接进行字节读写即可。

现在**内核态**可以直接使用虚拟GPU设备。但是想要在用户态使用(设计应用)还需要系统调用。

```rust
pub fn sys_framebuffer() -> isize {
    let fb = GPU_DEVICE.get_framebuffer();
    let len = fb.len();
    // println!("[kernel] FrameBuffer: addr 0x{:X}, len {}", fb.as_ptr() as usize , len);
    let fb_start_pa = PhysAddr::from(fb.as_ptr() as usize);
    assert!(fb_start_pa.aligned());
    let fb_start_ppn = fb_start_pa.floor();
    let fb_start_vpn = VirtAddr::from(FB_VADDR).floor();
    let pn_offset = fb_start_ppn.0 as isize - fb_start_vpn.0 as isize;

    let current_process = current_process();
    let mut inner = current_process.inner_exclusive_access(file!(), line!());
    inner.memory_set.push(
        MapArea::new(
            (FB_VADDR as usize).into(),
            (FB_VADDR + len as usize).into(),
            MapType::Linear(pn_offset),
            MapPermission::R | MapPermission::W | MapPermission::U,
        ),
        None,
    );
    FB_VADDR as isize
}

pub fn sys_framebuffer_flush() -> isize {
    GPU_DEVICE.flush();
    0
}
```

这两个系统调用分别对应了两个步骤：**获取帧地址并尝试渲染**和**刷新帧**



#### 移植图形库辅助开发

图形库**embedded-graphics**为图形化的开发提供很多便利，仅需要实现`trait Display`即可方便作画

```rust
pub struct Display {
    pub size: Size,
    pub fb: &'static mut [u8],
}

impl Display {
    pub fn new(size: Size) -> Self {
        let fb_ptr = framebuffer() as *mut u8;
        let fb = unsafe { core::slice::from_raw_parts_mut(fb_ptr, VIRTGPU_LEN as usize) };
        Self { size, fb }
    }
    pub fn framebuffer(&mut self) -> &mut [u8] {
        self.fb
    }
    pub fn paint_on_framebuffer(&mut self, p: impl FnOnce(&mut [u8]) -> ()) {
        p(self.framebuffer());
    }
    pub fn flush(&self) {
        framebuffer_flush();
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        self.size
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|px| {
            let idx = (px.0.y * VIRTGPU_XRES as i32 + px.0.x) as usize * 4;
            if idx + 2 >= self.fb.len() {
                return;
            }
            self.fb[idx] = px.1.b();
            self.fb[idx + 1] = px.1.g();
            self.fb[idx + 2] = px.1.r();
        });
        Ok(())
    }
}
```

这样，就可以通过该图形库辅助图形化开发。

## 参考资料

1. https://rcore-os.cn/rCore-Tutorial-Book-v3/
2. https://github.com/isrc-cas/riscv-isa-manual-cn

3. https://rustmagazine.github.io/rust_magazine_2021/