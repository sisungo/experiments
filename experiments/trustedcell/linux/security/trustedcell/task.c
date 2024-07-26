/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Task hooks
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/cred.h>
#include <uapi/linux/trustedcell.h>
#include <linux/string.h>
#include <linux/vmalloc.h>

#include "include/setup.h"
#include "include/cred.h"
#include "include/enforce_queue.h"
#include "include/task.h"

static int hook_task_kill(struct task_struct *p, struct kernel_siginfo *info,
           int sig, const struct cred *cred)
{
  return 0;
}

static int hook_ptrace_access_check(struct task_struct *const child,
           const unsigned int mode)
{
  struct trustedcell_id *current_cell_id = trustedcell_get_current_cell_id();
  struct trustedcell_id *child_cell_id = trustedcell_cred(child->cred)->cell_id;

  if (!current_cell_id) {
    return 0;
  }
  if (!child_cell_id) {
    return -EACCES;
  }
  if (strcmp(current_cell_id->str, child_cell_id->str) == 0) {
    return 0;
  }
  return -EACCES;
}

static struct security_hook_list trustedcell_hooks[] __ro_after_init = {
  LSM_HOOK_INIT(task_kill, hook_task_kill),

  LSM_HOOK_INIT(ptrace_access_check, hook_ptrace_access_check),
};

__init void trustedcell_add_task_hooks(void)
{
  security_add_hooks(trustedcell_hooks, ARRAY_SIZE(trustedcell_hooks),
      &trustedcell_lsmid);
}
