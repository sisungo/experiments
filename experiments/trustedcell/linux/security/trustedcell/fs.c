/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Filesystem hooks
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/cred.h>
#include <uapi/linux/trustedcell.h>
#include <linux/string.h>
#include <linux/vmalloc.h>

#include "include/setup.h"
#include "include/cred.h"
#include "include/fs.h"
#include "include/enforce_queue.h"
#include "include/util.h"

static const char *get_file_open_action(struct file *const file)
{
	unsigned int code = file->f_mode & (FMODE_READ | FMODE_WRITE);

	if (S_ISDIR(file_inode(file)->i_mode)) {
		return "posix.read_dir";
	}
	switch (code) {
	case FMODE_READ:
		return "posix.open_ro";
	case FMODE_WRITE:
		return "posix.open_wo";
	case FMODE_READ | FMODE_WRITE:
		return "posix.open_rw";
	default:
		return "posix.open";
	}
}

static int hook_inode_init_security(struct inode *const inode,
				    struct inode *dir, const struct qstr *qstr,
				    struct xattr *xattrs, int *xattr_count)
{
	struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
	struct xattr *category = lsm_get_xattr_slot(xattrs, xattr_count);
	struct xattr *owner = lsm_get_xattr_slot(xattrs, xattr_count);
	const char *owner_str;
	if (category) {
		category->value =
			kstrdup(trustedcell_inode(dir)->category, GFP_NOFS);
		if (!category->value) {
			return -ENOMEM;
		}
		category->value_len = strlen(category->value);
		category->name = TRUSTEDCELL_XATTR_CATEGORY_SUFFIX;
	}
	if (!cell_id) {
		owner_str = trustedcell_inode(dir)->owner;
	} else {
		owner_str = cell_id->str;
	}
	if (owner) {
		owner->value = kstrdup(owner_str, GFP_NOFS);
		if (!owner->value) {
			return -ENOMEM;
		}
		owner->value_len = strlen(owner_str);
		owner->name = TRUSTEDCELL_XATTR_OWNER_SUFFIX;
	}
	return 0;
}

static int hook_inode_create(struct inode *dir, struct dentry *dentry,
			     umode_t mode)
{
	struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
	if (!cell_id) {
		return 0;
	}
	return trustedcell_decide(trustedcell_get_current_initial_uid(),
				  cell_id, trustedcell_inode(dir)->category,
				  trustedcell_inode(dir)->owner,
				  "posix.create_reg", GFP_KERNEL);
}

static int hook_inode_unlink(struct inode *dir, struct dentry *dentry)
{
	struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
	if (!cell_id) {
		return 0;
	}
	return trustedcell_decide(trustedcell_get_current_initial_uid(),
				  cell_id,
				  trustedcell_inode(d_inode(dentry))->category,
				  trustedcell_inode(d_inode(dentry))->owner,
				  "posix.unlink", GFP_KERNEL);
}

static int hook_inode_mkdir(struct inode *dir, struct dentry *dentry,
			    umode_t mode)
{
	struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
	if (!cell_id) {
		return 0;
	}
	return trustedcell_decide(trustedcell_get_current_initial_uid(),
				  cell_id, trustedcell_inode(dir)->category,
				  trustedcell_inode(dir)->owner, "posix.mkdir",
				  GFP_KERNEL);
}

static int hook_inode_rmdir(struct inode *dir, struct dentry *dentry)
{
	struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
	if (!cell_id) {
		return 0;
	}
	return trustedcell_decide(trustedcell_get_current_initial_uid(),
				  cell_id,
				  trustedcell_inode(d_inode(dentry))->category,
				  trustedcell_inode(d_inode(dentry))->owner,
				  "posix.rmdir", GFP_KERNEL);
}

static int hook_inode_mknod(struct inode *dir, struct dentry *dentry,
			    umode_t mode, dev_t dev)
{
	struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
	if (!cell_id) {
		return 0;
	}
	return trustedcell_decide(trustedcell_get_current_initial_uid(),
				  cell_id, trustedcell_inode(dir)->category,
				  trustedcell_inode(dir)->owner, "posix.mknod",
				  GFP_KERNEL);
}

static int hook_inode_setxattr(struct mnt_idmap *mnt_idmap,
			       struct dentry *dentry, const char *name,
			       const void *value, size_t size, int flags)
{
	if (strcmp(name, TRUSTEDCELL_XATTR_CATEGORY) == 0) {
		if (size > TRUSTEDCELL_CATEGORY_MAX || size == 0 ||
		    value == NULL) {
			return -EINVAL;
		}
	} else if (strcmp(name, TRUSTEDCELL_XATTR_OWNER) == 0) {
		if (size > TRUSTEDCELL_CELL_IDENTIFIER_MAX || size == 0 ||
		    value == NULL) {
			return -EINVAL;
		}
	}
	return 0;
}

static void hook_inode_post_setxattr(struct dentry *dentry, const char *name,
				     const void *value, size_t size, int flags)
{
	if (strcmp(name, TRUSTEDCELL_XATTR_CATEGORY) == 0) {
		memcpy(trustedcell_inode(d_inode(dentry))->category, value,
		       size);
		trustedcell_inode(d_inode(dentry))->category[size] = '\0';
	} else if (strcmp(name, TRUSTEDCELL_XATTR_OWNER) == 0) {
		memcpy(trustedcell_inode(d_inode(dentry))->owner, value, size);
		trustedcell_inode(d_inode(dentry))->owner[size] = '\0';
	}
}

static void hook_d_instantiate(struct dentry *opt_dentry, struct inode *inode)
{
	int status;
	char *category = trustedcell_inode(inode)->category;
	char *owner = trustedcell_inode(inode)->owner;
	struct super_block *sb = inode->i_sb;

	if (trustedcell_inode(inode)->flags & TRUSTEDCELL_INODE_INITIALIZED) {
		return;
	}
	if (!(inode->i_opflags & IOP_XATTR)) {
		if (trustedcell_sb(sb)->flags & TRUSTEDCELL_SB_INITIALIZED) {
			strcpy(category, trustedcell_sb(sb)->category);
			strcpy(owner, trustedcell_sb(sb)->owner);
		} else {
			strcpy(category, "?");
			strcpy(owner, "?");
		}
		return;
	}
	status = __vfs_getxattr(opt_dentry, inode, TRUSTEDCELL_XATTR_CATEGORY,
				category, TRUSTEDCELL_CATEGORY_MAX);
	if (status <= 0) {
		if (IS_ROOT(opt_dentry)) {
			strcpy(category, "?");
		} else {
			strcpy(category,
			       trustedcell_inode(d_inode(opt_dentry->d_parent))
				       ->category);
		}
	}
	status = __vfs_getxattr(opt_dentry, inode, TRUSTEDCELL_XATTR_OWNER,
				owner, TRUSTEDCELL_CELL_IDENTIFIER_MAX);
	if (status <= 0) {
		strcpy(owner, "?");
	}
	trustedcell_inode(inode)->flags |= TRUSTEDCELL_INODE_INITIALIZED;
}

static int hook_file_open(struct file *const file)
{
	struct trustedcell_id *cell_id =
		trustedcell_cred(file->f_cred)->cell_id;

	if (!cell_id) {
		return 0;
	}

	return trustedcell_decide(file->f_cred->uid, cell_id,
				  trustedcell_inode(file_inode(file))->category,
				  trustedcell_inode(file_inode(file))->owner,
				  get_file_open_action(file), GFP_KERNEL);
}

static void hook_task_to_inode(struct task_struct *p, struct inode *inode)
{
	struct trustedcell_id *cell_id = trustedcell_cred(p->cred)->cell_id;
	if (cell_id) {
		strcpy(trustedcell_inode(inode)->owner, cell_id->str);
	}
	strcpy(trustedcell_inode(inode)->category, TRUSTEDCELL_PROC_CATEGORY);
}

static struct security_hook_list trustedcell_hooks[] __ro_after_init = {
	LSM_HOOK_INIT(inode_init_security, hook_inode_init_security),
	LSM_HOOK_INIT(inode_create, hook_inode_create),
	LSM_HOOK_INIT(inode_unlink, hook_inode_unlink),
	LSM_HOOK_INIT(inode_mkdir, hook_inode_mkdir),
	LSM_HOOK_INIT(inode_mknod, hook_inode_mknod),
	LSM_HOOK_INIT(inode_rmdir, hook_inode_rmdir),
	LSM_HOOK_INIT(inode_setxattr, hook_inode_setxattr),
	LSM_HOOK_INIT(inode_post_setxattr, hook_inode_post_setxattr),

	LSM_HOOK_INIT(d_instantiate, hook_d_instantiate),

	LSM_HOOK_INIT(file_open, hook_file_open),

	LSM_HOOK_INIT(task_to_inode, hook_task_to_inode),
};

__init void trustedcell_add_fs_hooks(void)
{
	security_add_hooks(trustedcell_hooks, ARRAY_SIZE(trustedcell_hooks),
			   &trustedcell_lsmid);
}
