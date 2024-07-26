/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Enforce Caches
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _SECURITY_TRUSTEDCELL_CACHE_H
#define _SECURITY_TRUSTEDCELL_CACHE_H

#include <linux/uidgid_types.h>

#include "util.h"

#define TRUSTEDCELL_ENFORCE_CACHE_SLOTS 256
#define TRUSTEDCELL_ENFORCE_CACHE_SLOT_SIZE 64

struct trustedcell_enforce_cache_item {
  uid_t uid;
  struct trustedcell_id *cell;
  const char *category;
  const char *owner;
  const char *action;
  int resp;
};

void trustedcell_enforce_cache_init(void);
int trustedcell_enforce_cache_add(struct trustedcell_enforce_cache_item item);
int trustedcell_enforce_cache_match(struct trustedcell_enforce_cache_item item);

#endif /* _SECURITY_TRUSTEDCELL_CACHE_H */
