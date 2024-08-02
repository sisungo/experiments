/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Credentials
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/cred.h>
#include <uapi/linux/trustedcell.h>
#include <linux/string.h>
#include <linux/vmalloc.h>

#include "include/setup.h"
#include "include/cred.h"
#include "include/util.h"

bool trustedcell_check_cell_identifier(const char *cell_identifier)
{
	if (*cell_identifier == 0) {
		return false;
	}
	while (*cell_identifier) {
		if (!isgraph(*cell_identifier)) {
			return false;
		}
		cell_identifier++;
	}
	return true;
}

static int hook_cred_prepare(struct cred *const new,
			     const struct cred *const old, const gfp_t gfp)
{
	*trustedcell_cred(new) = *trustedcell_cred(old);

	struct trustedcell_id *identifier = trustedcell_cred(new)->cell_id;
	if (identifier) {
		trustedcell_get_id(identifier);
	}

	return 0;
}

static void hook_cred_free(struct cred *const cred)
{
	struct trustedcell_id *identifier = trustedcell_cred(cred)->cell_id;
	if (identifier) {
		trustedcell_put_id(identifier);
	}
}

static struct security_hook_list trustedcell_hooks[] __ro_after_init = {
	LSM_HOOK_INIT(cred_prepare, hook_cred_prepare),
	LSM_HOOK_INIT(cred_free, hook_cred_free),
};

__init void trustedcell_add_cred_hooks(void)
{
	security_add_hooks(trustedcell_hooks, ARRAY_SIZE(trustedcell_hooks),
			   &trustedcell_lsmid);
}
