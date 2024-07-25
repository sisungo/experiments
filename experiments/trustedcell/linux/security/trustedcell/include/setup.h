/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Security framework setup
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _SECURITY_TRUSTEDCELL_SETUP_H
#define _SECURITY_TRUSTEDCELL_SETUP_H

#include <linux/lsm_hooks.h>
#include <linux/types.h>

extern bool trustedcell_initialized;

extern struct lsm_blob_sizes trustedcell_blob_sizes;
extern const struct lsm_id trustedcell_lsmid;
extern pid_t trustedcell_host_tgid;

#endif /* _SECURITY_TRUSTEDCELL_SETUP_H */
