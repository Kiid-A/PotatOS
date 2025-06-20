北京邮电大学2025春季操作系统课程设计 <br>
## 🥔什么是PotatOS <br>

PotatOS是一个用 Rust 编写的基于 [2024 春夏季开源操作系统训练营 rCore 项目](https://github.com/rcore-os)的 RISC-V 架构的兼容 POSIX 协议的操作系统内核，实现了基本的进程管理、内存管理、文件系统和设备驱动。

## 使用方法

### 下载Rust

查看[官方教程](https://www.rust-lang.org/tools/install)。

```sh
$ rustup target add riscv64gc-unknown-none-elf
$ cargo install cargo-binutils --vers =0.3.3
$ rustup component add llvm-tools-preview
$ rustup component add rust-src
```

### 下载Qemu

```sh
# install dependency packages
$ sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev git tmux python3 python3-pip
# download Qemu source code
$ wget https://download.qemu.org/qemu-7.0.0.tar.xz
# extract to qemu-7.0.0/
$ tar xvJf qemu-7.0.0.tar.xz
$ cd qemu-7.0.0
# build
$ ./configure --target-list=riscv64-softmmu,riscv64-linux-user
$ make -j$(nproc)
```

载入环境变量：

```
export PATH=$PATH:/path/to/qemu-7.0.0
export PATH=$PATH:/path/to/qemu-7.0.0/riscv64-softmmu
export PATH=$PATH:/path/to/qemu-7.0.0/riscv64-linux-user
```

更新shell：

```sh
$ source ~/.bashrc
```

检查QEMU版本

```sh
$ qemu-system-riscv64 --version
QEMU emulator version 7.0.0
Copyright (c) 2003-2020 Fabrice Bellard and the QEMU Project developers
```

### 下载工具链-RISC-V GNU Embedded Toolchain(including GDB)

根据机器下载对应版本：[官方网站](https://www.sifive.com/software)(Ctrl+F 'toolchain').

加入环境变量

检查GDB版本：

```sh
$ riscv64-unknown-elf-gdb --version
GNU gdb (SiFive GDB-Metal 10.1.0-2020.12.7) 10.1
Copyright (C) 2020 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
```

### 运行程序

```sh
$ git clone https://github.com/rcore-os/rCore-Tutorial-v3.git
$ cd PotatOS/os
$ make run
```
