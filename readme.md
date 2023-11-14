# sys3

To build this project, you'll need several things:
- The most recent version of LLVM/Clang, which you may obtain here: https://llvm.org/docs/GettingStarted.html
- CMake to build LLVM, which you may obtain here: https://cmake.org/
- The `cargo-make` tool, which you may obtain here: https://github.com/sagiegurari/cargo-make

---

***NOTE: the above details serve only for posterity as of 2023.11***

Trident is an operating system originally developed as part of a series on [Medium](https://medium.com/@zaiqi) and [MBP2](https://mbp2.blog/@az)
before being co-opted into a personal research project.

As of 2023.11, you will need:
- The Rust nightly as of 2023.11.12, which you may obtain through the official website: https://rust-lang.org/learn/get-started
- Docker to run the build environment: https://www.docker.com/get-started/

# Trident

VERSION: 3.0.1/EARLY/UNRELEASED  
LICENSE: [Apache-2.0](https://github.com/azyklus/sys3/blob/trunk/LICENSE)  
README: [Where would you rather be?](https://xkcd.com/650/)  
INFO:  

The upstream kernel, bootloader, and lib(s) source tree can be found here.  

### Usage

TODO: Update this section when userland modules are created.

### Install

When using this system, it is recommended to install to a virtual machine,
as the kernel is not ready for any level of normal usage.

To install the kernel to your system, you'll need to build it from source
and launch it from QEMU or another virtual machine:

```
git clone https://github.com/azyklus/sys3.git
cd sys3

cargo build # TODO: Update with proper build process
```

### Development

You will eventually be able to develop your own extensions to Trident 3 through a proposed Extensions API.

### Contributing

If you'd like to contribute to this project, please [fork](https://github.com/azyklus/sys3/fork) it and
submit pull requests with your desired features.

1. [Fork it.](https://github.com/azyklus/sys3/fork)  
2. ????? (I forgot what went here)  
3. Submit pull request with your feature. ("[FEATURE] describe your feature").  
4. Profit?  

### Useful Links

- [MBP2 Page](https://mbp2.blog/src/@trident)
- [Matrix Room](https://matrix.to/#/%23two-worlds:mozilla.org)
- [Discord Server](https://discord.gg/B9agTdVH4U)
