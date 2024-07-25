/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Enforce Queue
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _SECURITY_TRUSTEDCELL_ENFORCE_QUEUE_H
#define _SECURITY_TRUSTEDCELL_ENFORCE_QUEUE_H

#include <linux/wait.h>
#include <linux/types.h>
#include <linux/vmalloc.h>

#include "util.h"

struct trustedcell_request {
  int64_t request_id;
  kuid_t uid;
  struct trustedcell_id *cell;
  const char *category;
  const char *owner;
  const char *action;
};

int trustedcell_send_request(struct trustedcell_request request);
int trustedcell_recv_request(struct trustedcell_request *request);
int trustedcell_put_response(int64_t request_id, int permit);
int trustedcell_invoke_host(kuid_t uid, struct trustedcell_id *cell,
    const char *category, const char *owner, const char *action,
    gfp_t gfp);

static inline void trustedcell_free_request(struct trustedcell_request request)
{
  trustedcell_put_id(request.cell);
  kfree(request.category);
  kfree(request.owner);
  kfree(request.action);
}

#endif /* _SECURITY_TRUSTEDCELL_ENFORCE_QUEUE_H */
