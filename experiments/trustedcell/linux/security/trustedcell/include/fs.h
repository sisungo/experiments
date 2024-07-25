/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Filesystem hooks
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#ifndef _SECURITY_TRUSTEDCELL_FS_H
#define _SECURITY_TRUSTEDCELL_FS_H

#include <linux/types.h>
#include <uapi/linux/trustedcell.h>

#include "setup.h"
#include "util.h"

#define TRUSTEDCELL_INODE_INITIALIZED 8

struct trustedcell_inode_security {
  char category[TRUSTEDCELL_CATEGORY_MAX];
  char owner[TRUSTEDCELL_CELL_IDENTIFIER_MAX];
  uint32_t flags;
};

__init void trustedcell_add_fs_hooks(void);

static inline struct trustedcell_inode_security *
trustedcell_inode(const struct inode *const inode)
{
  return inode->i_security + trustedcell_blob_sizes.lbs_inode;
}

#endif /* _SECURITY_TRUSTEDCELL_FS_H */
