/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Credentials
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _SECURITY_TRUSTEDCELL_CRED_H
#define _SECURITY_TRUSTEDCELL_CRED_H

#include <linux/cred.h>
#include <uapi/linux/trustedcell.h>

#include "setup.h"
#include "util.h"

struct trustedcell_cred_security {
	kuid_t initial_uid;
	struct trustedcell_id *cell_id;
};

static inline struct trustedcell_cred_security *
trustedcell_cred(const struct cred *cred)
{
	return cred->security + trustedcell_blob_sizes.lbs_cred;
}

static inline struct trustedcell_id *trustedcell_get_current_cell_id(void)
{
	return trustedcell_cred(current_cred())->cell_id;
}

static inline kuid_t trustedcell_get_current_initial_uid(void)
{
	return trustedcell_cred(current_cred())->initial_uid;
}

bool trustedcell_check_cell_identifier(const char *cell_identifier);

__init void trustedcell_add_cred_hooks(void);

#endif /* _SECURITY_TRUSTEDCELL_CRED_H */
