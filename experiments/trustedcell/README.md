# TrustedCell
TrustedCell is a security mechanism that provides application-based, dynamic access control for Linux desktop, implemented
as a Linux Security Module.

## Screenshots
See [screenshots](screenshots).

## Core Principles
A working TrustedCell environment consists of two parts: the kernel-space LSM and the userspace "host" server. The kernel-space
LSM records subject and object contexts, and then requests the userspace "host" server to judge if the subject has access to
the object. When invoking the host for access checking, the syscall is interruptibly blocked until the host responded. The
enforce decision is then cached, to avoid frequently requesting the host and improve performance.

Focusing on desktop usage, TrustedCell is not aiming to confine all processes running on the system. Instead, a process confines
itself by telling the LSM who they are. A confined environment is called a `cell`, uniquely identified by a string called
`cell identifier`. When a process is confined in a cell, all its children are confined in the same cell, too. Changing cell is
possible, however, requires permission from the host.

A subject security context is composed of an Initial UID\(Effective UID when the cell is entered\) and a Cell ID\(Unique Cell 
Identifier\).

An object security context is composed of a category and an owner. An owner is a cell identifier that tells which cell the
object was created in, and a category tells the object's security type. When fetching an object's owner, the LSM will only
attempt to fetch the object itself's xattrs. When fetching the object's category, the LSM will walk the path, until the nearest
xattr was found. For example, when deciding category of `/home/somebody/Music/music.opus`, the road may be:

 - xattr `security.tc_category` of `/home/somebody/Music/music.opus` == `-ENODATA`: try fetching from parent;
 - xattr `security.tc_category` of `/home/somebody/Music` == `-ENODATA`: try fetching from parent;
 - xattr `security.tc_category` of `/home/somebody` == `"user_home"`: return `"user_home"`.

Kernel APIs are used, in order to avoid TOCTOU vulnerabilities. If category's name starts `~`, the category is an Owner-Aware
Category. Generally, access checks are category-only, however, owners are in consideration if the category is owner-aware. This
convention affects caching.

An userspace host server opens `/sys/kernel/security/trustedcell/host` to handle TrustedCell requests. The file can only be
opened by one process\(task group\) at the same time and the process must run as `root` when opening the file, for security
reasons. The host server may want to show permission dialog to the user's desktop session: this can be done by having helper
daemons for each user session.

For performance, the design goal is to minimize the cost of invoking userspace by effectively caching enforcing decisions.
Currently, a cache of the LRU algorithm is used, and the performance impact is to be evaluated.
