/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - SecurityFS interface
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/security.h>
#include <linux/vmalloc.h>
#include <linux/spinlock.h>
#include <uapi/asm-generic/errno.h>

#include "include/setup.h"
#include "include/trustedcell_secfs.h"
#include "include/cred.h"
#include "include/enforce_queue.h"
#include "include/util.h"

DEFINE_SPINLOCK(trustedcell_host_lock);

static ssize_t secfs_status_read(struct file *file, char __user *ubuf,
    size_t count, loff_t *ppos)
{
  char status = READ_ONCE(trustedcell_host_tgid) ? '1' : '0';
  return simple_read_from_buffer(ubuf, count, ppos, &status, 1);
}

static ssize_t secfs_me_read(struct file *file, char __user *ubuf,
    size_t count, loff_t *ppos)
{
  if (!trustedcell_initialized) {
    return -EOPNOTSUPP;
  }
  struct trustedcell_id *cell_id = trustedcell_get_current_cell_id();
  const char *buffer = cell_id ? cell_id->str : "";
  return simple_read_from_buffer(ubuf, count, ppos, buffer, strlen(buffer));
}

static ssize_t secfs_me_write(struct file *file, const char __user *ubuf,
    size_t count, loff_t *ppos)
{
  int status;
  struct trustedcell_id *cell_id;
  char *cell_id_str;
  struct cred *new_cred;

  if (!trustedcell_initialized) {
    return -EOPNOTSUPP;
  }
  if (count > TRUSTEDCELL_CELL_IDENTIFIER_MAX) {
    return -EINVAL;
  }
  cell_id_str = kzalloc(count + 1, GFP_KERNEL);
  if (!cell_id_str) {
    return -ENOMEM;
  }
  if (copy_from_user(cell_id_str, ubuf, count)) {
    status = -EINVAL;
    goto out_free_cell_id_str;
  }
  if (!trustedcell_check_cell_identifier(cell_id_str))
  {
    status = -EINVAL;
    goto out_free_cell_id_str;
  }
  struct trustedcell_id *current_cell_id = trustedcell_get_current_cell_id();
  if (current_cell_id) {
    status = trustedcell_decide(current_uid(), current_cell_id,
        "~trustedcell", cell_id_str, "trustedcell.change_cell", GFP_KERNEL);
    if (status < 0 && strcmp(cell_id_str, current_cell_id->str) != 0) {
      goto out_free_cell_id_str;
    }
  }
  cell_id = kmalloc(sizeof(struct trustedcell_id), GFP_KERNEL);
  if (!cell_id) {
    status = -ENOMEM;
    goto out_free_cell_id_str;
  }
  cell_id->str = cell_id_str;
  trustedcell_init_id(cell_id);
  new_cred = prepare_creds();
  if (!new_cred) {
    status = -ENOMEM;
    goto out_put_cell_id;
  }
  trustedcell_cred(new_cred)->cell_id = cell_id;
  if ((status = commit_creds(new_cred)) < 0) {
    goto out_put_cell_id;
  }
  return count;

out_free_cell_id_str:
  kfree(cell_id_str);
  return status;

out_put_cell_id:
  trustedcell_put_id(cell_id);
  return status;
}

static int secfs_host_open(struct inode *inode, struct file *file)
{
  if (!trustedcell_initialized) {
    return -EOPNOTSUPP;
  }
  if (trustedcell_get_current_cell_id()) {
    return -EACCES;
  }
  spin_lock(&trustedcell_host_lock);
  pid_t current_host_tgid = READ_ONCE(trustedcell_host_tgid);
  if (current_host_tgid != 0 && current->tgid != current_host_tgid) {
    spin_unlock(&trustedcell_host_lock);
    return -EBUSY;
  }
  WRITE_ONCE(trustedcell_host_tgid, current->tgid);
  spin_unlock(&trustedcell_host_lock);
  return 0;
}

static int secfs_host_release(struct inode *inode, struct file *file)
{
  WRITE_ONCE(trustedcell_host_tgid, 0);
  return 0;
}

static ssize_t secfs_host_read(struct file *file, char __user *ubuf,
    size_t count, loff_t *ppos)
{
  int status;
  struct trustedcell_request request;
  char buffer[512];

  if (count < 512) {
    return -EINVAL;
  }

  while ((status = trustedcell_recv_request(&request)) < 0) {
    if (signal_pending(current)) {
      return -ERESTARTSYS;
    }
  }
  snprintf(buffer, sizeof(buffer), "%lld %d %s %s %s %s", request.request_id,
      from_kuid(current_user_ns(), request.uid),
      request.cell->str, request.category, request.owner,
      request.action);
  size_t len = strlen(buffer);
  status = copy_to_user(ubuf, buffer, len);
  trustedcell_free_request(request);
  return (status < 0) ? status : len;
}

static ssize_t secfs_host_write(struct file *file, const char __user *ubuf,
    size_t count, loff_t *ppos)
{
  int status;
  int bytes_written;
  int64_t request_id;
  int permit;
  int cachable;
  char buf[80];

  if (count > sizeof(buf) - 1) {
    return -EINVAL;
  }
  memset(buf, 0, sizeof(buf));
  bytes_written = copy_from_user(buf, ubuf, count);
  if (bytes_written < 0) {
    return bytes_written;
  }
  if (sscanf(buf, "%lld %d %d", &request_id, &permit, &cachable) < 0) {
    return -EINVAL;
  }
  status = trustedcell_put_response(request_id, permit, !!cachable);
  return status < 0 ? status : bytes_written;
}

static const struct file_operations secfs_status_ops = {
  .read = secfs_status_read,
};

static const struct file_operations secfs_me_ops = {
  .read = secfs_me_read,
  .write = secfs_me_write,
};
static const struct file_operations secfs_host_ops = {
  .open = secfs_host_open,
  .release = secfs_host_release,
  .read = secfs_host_read,
  .write = secfs_host_write,
};

static int __init trustedcell_secfs_init(void)
{
  struct dentry *directory = securityfs_create_dir("trustedcell", NULL);
  if (!directory) {
    return -1;
  }
  securityfs_create_file("status", 0666, directory, NULL,
      &secfs_status_ops);
  securityfs_create_file("me", 0666, directory, NULL,
      &secfs_me_ops);
  securityfs_create_file("host", 0600, directory, NULL,
      &secfs_host_ops);
  return 0;
}

core_initcall(trustedcell_secfs_init);
