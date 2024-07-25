/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Shared utilities
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _SECURITY_TRUSTEDCELL_UTIL_H
#define _SECURITY_TRUSTEDCELL_UTIL_H

#include <linux/kref.h>
#include <linux/vmalloc.h>

struct trustedcell_id {
  const char *str;
  struct kref refcount;
};

static inline void trustedcell_init_id(struct trustedcell_id *data)
{
  kref_init(&data->refcount);
}

static inline void trustedcell_get_id(struct trustedcell_id *data)
{
  kref_get(&data->refcount);
}

void trustedcell_put_id(struct trustedcell_id *data);

int trustedcell_decide(kuid_t uid, struct trustedcell_id *cell,
    const char *category, const char *owner,
    const char *action, gfp_t gfp);

#endif /* _SECURITY_TRUSTEDCELL_UTIL_H */
