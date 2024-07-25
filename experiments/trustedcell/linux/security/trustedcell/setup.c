// SPDX-License-Identifier: GPL-2.0-only
/* 
 * TrustedCell LSM - Security framework setup
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/init.h>
#include <linux/lsm_hooks.h>
#include <uapi/linux/lsm.h>
#include <linux/types.h>

#include "include/common.h"
#include "include/cred.h"
#include "include/fs.h"
#include "include/cache.h"
#include "include/task.h"

bool trustedcell_initialized __ro_after_init = false;
pid_t trustedcell_host_tgid = 0;

struct lsm_blob_sizes trustedcell_blob_sizes __ro_after_init = {
  .lbs_cred = sizeof(struct trustedcell_cred_security),
  .lbs_inode = sizeof(struct trustedcell_inode_security),
  .lbs_xattr_count = 1,
};

const struct lsm_id trustedcell_lsmid = {
  .name = "trustedcell",
  .id = LSM_ID_TRUSTEDCELL,
};

static int __init trustedcell_init(void)
{
  trustedcell_enforce_cache_init();
  trustedcell_add_cred_hooks();
  trustedcell_add_task_hooks();
  trustedcell_add_fs_hooks();
  trustedcell_initialized = true;
  pr_info("Kernel-space initialized.\n");
  return 0;
}

DEFINE_LSM(TRUSTEDCELL_NAME) = {
  .name = "trustedcell",
  .init = trustedcell_init,
  .blobs = &trustedcell_blob_sizes,
};
