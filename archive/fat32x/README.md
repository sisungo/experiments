# FAT32x

FAT32x is a wrapper around FAT32/exFAT filesystem to provide unsupported features, like:

 * Large Files \(> 4GiB\)
 * Symbolic Links
 * Unix Permissions
 * more...

On Linux, FAT32x is present as a FUSE filesystem that can be mounted. FAT32x itself is also a library \(written in `#![no_std]`
Rust\), so it can be easily ported to other systems, like porting it to an out-of-tree kernel module.

## Frequently Asked Questions

### Is FAT32x Safe To Use?
By default, FAT32x doesn't directly read/write from/to your disk. Instead, it works on the top of your kernel's filesystem 
driver, so it will probably not eat your data, though its implementation has bugs.

### Should I Use FAT32x?
FAT32x is designed for compatibility with systems that support FAT32/exFAT only. If modern filesystems like `ext4` are available,
use them, not FAT32x.
