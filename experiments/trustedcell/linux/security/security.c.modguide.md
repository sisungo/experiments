Modify definition of `LSM_CONFIG_COUNT` in `security.c`. This is probably near line 55. The modified version should look like:

```c
/*
 * How many LSMs are built into the kernel as determined at
 * build time. Used to determine fixed array sizes.
 * The capability module is accounted for by CONFIG_SECURITY
 */
#define LSM_CONFIG_COUNT ( \
	(IS_ENABLED(CONFIG_SECURITY) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_SELINUX) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_SMACK) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_TOMOYO) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_APPARMOR) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_YAMA) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_LOADPIN) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_SAFESETID) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_LOCKDOWN_LSM) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_BPF_LSM) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_SECURITY_LANDLOCK) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_IMA) ? 1 : 0) + \
	(IS_ENABLED(CONFIG_EVM) ? 1 : 0) + \
    (IS_ENABLED(CONFIG_SECURITY_TRUSTEDCELL) ? 1 : 0))
```

The last line with keyword `TRUSTEDCELL` is required and added by TrustedCell. Take care of brackets.