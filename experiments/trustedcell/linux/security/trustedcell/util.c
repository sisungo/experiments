// SPDX-License-Identifier: GPL-2.0-only
/* 
 * TrustedCell LSM - Shared utilities
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/init.h>
#include <linux/lsm_hooks.h>
#include <uapi/linux/lsm.h>
#include <linux/types.h>

#include "include/enforce_queue.h"
#include "include/cache.h"
#include "include/util.h"

static void release_id(struct kref *ref)
{
	struct trustedcell_id *data =
		container_of(ref, struct trustedcell_id, refcount);
	kfree(data->str);
	kfree(data);
}

void trustedcell_put_id(struct trustedcell_id *data)
{
	kref_put(&data->refcount, release_id);
}

int trustedcell_decide(kuid_t uid, struct trustedcell_id *cell,
		       const char *category, const char *owner,
		       const char *action, gfp_t gfp)
{
	int status;
	bool cachable;
	struct trustedcell_enforce_cache_item item = {
		.uid = uid.val,
		.cell = cell,
		.category = category,
		.owner = owner,
		.action = action,
		.resp = -EACCES,
	};
	if ((status = trustedcell_enforce_cache_match(item)) != -ENODATA) {
		return status;
	}
	status = trustedcell_invoke_host(&cachable, uid, cell, category, owner,
					 action, gfp);
	if (!cachable) {
		return status;
	}
	item.cell = cell;
	item.category = kstrdup(category, gfp);
	if (!item.category) {
		goto out_free_category;
	}
	item.owner = kstrdup(owner, gfp);
	if (!item.owner) {
		goto out_free_owner;
	}
	item.action = kstrdup(action, gfp);
	if (!item.action) {
		goto out_free_action;
	}
	item.resp = status;
	trustedcell_get_id(cell);
	if ((status = trustedcell_enforce_cache_add(item)) < 0) {
		goto out_put_cell;
	}
	return item.resp;

out_put_cell:
	trustedcell_put_id(cell);
out_free_action:
	kfree(item.action);
out_free_owner:
	kfree(item.owner);
out_free_category:
	kfree(item.category);
	return -ENOMEM;
}
