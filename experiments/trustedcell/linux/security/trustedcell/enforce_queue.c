/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Enforce Queue
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/init.h>
#include <linux/wait.h>
#include <linux/types.h>
#include <linux/atomic.h>
#include <linux/kfifo.h>
#include <linux/hashtable.h>

#include "include/enforce_queue.h"

DECLARE_WAIT_QUEUE_HEAD(trustedcell_request_wait_queue);
DEFINE_SPINLOCK(trustedcell_request_lock);
DEFINE_KFIFO(trustedcell_request_queue, struct trustedcell_request, 16);

DECLARE_WAIT_QUEUE_HEAD(trustedcell_response_wait_queue);
DECLARE_HASHTABLE(trustedcell_response_table, ilog2(16));
DEFINE_SPINLOCK(trustedcell_response_lock);

static atomic64_t next_request_id = ATOMIC64_INIT(1);

struct response_table_node {
  struct hlist_node hlist;
  int64_t request_id;
  int permit;
};

static inline int64_t trustedcell_pull_request_id(void)
{
  return atomic64_fetch_inc(&next_request_id);
}

int trustedcell_send_request(struct trustedcell_request request) {
  int status;
  status = wait_event_interruptible(trustedcell_request_wait_queue,
      !kfifo_is_full(&trustedcell_request_queue));
  if (status < 0) {
    return status;
  }
  spin_lock(&trustedcell_request_lock);
  kfifo_put(&trustedcell_request_queue, request);
  spin_unlock(&trustedcell_request_lock);
  wake_up_interruptible(&trustedcell_request_wait_queue);
  return 0;
}

int trustedcell_recv_request(struct trustedcell_request *request) {
  int status;

  status = wait_event_interruptible(trustedcell_request_wait_queue,
      !kfifo_is_empty(&trustedcell_request_queue));
  if (status < 0) {
    return status;
  }
  spin_lock(&trustedcell_request_lock);
  if (kfifo_get(&trustedcell_request_queue, request) != 1) {
    return -EWOULDBLOCK;
  }
  spin_unlock(&trustedcell_request_lock);
  wake_up_interruptible(&trustedcell_request_wait_queue);
  return 0;
}

int trustedcell_put_response(int64_t request_id, int permit, bool cachable)
{
  struct response_table_node *node;

  spin_lock(&trustedcell_response_lock);
  hash_for_each_possible(trustedcell_response_table, node, hlist, request_id) {
    if (node->request_id == request_id) {
      node->permit = (permit ? TRUSTEDCELL_GRANTED: 0)
        | (cachable ? TRUSTEDCELL_CACHABLE : 0);
      spin_unlock(&trustedcell_response_lock);
      wake_up_interruptible(&trustedcell_response_wait_queue);
      return 0;
    }
  }
  spin_unlock(&trustedcell_response_lock);
  return -EINVAL;
}

static struct response_table_node *
trustedcell_register_response(int64_t request_id)
{
  struct response_table_node *node 
    = kmalloc(sizeof(struct response_table_node), GFP_KERNEL);
  if (!node) {
    return NULL;
  }

  node->request_id = request_id;
  node->permit = -1;
  spin_lock(&trustedcell_response_lock);
  hash_add(trustedcell_response_table, &node->hlist, request_id);
  spin_unlock(&trustedcell_response_lock);
  return node;
}

static void trustedcell_unregister_response(struct response_table_node *node)
{
  spin_lock(&trustedcell_response_lock);
  hash_del(&node->hlist);
  spin_unlock(&trustedcell_response_lock);
  kfree(node);
}

static int trustedcell_get_response(int64_t request_id, bool *cachable)
{
  struct response_table_node *node;
  int permit;

	spin_lock(&trustedcell_response_lock);
	hash_for_each_possible(trustedcell_response_table, node, hlist, request_id) {
		if (node->request_id == request_id && node->permit != -1) {
			permit = !!(node->permit & TRUSTEDCELL_GRANTED);
      *cachable = !!(node->permit & TRUSTEDCELL_CACHABLE);
      hash_del(&node->hlist);
      kfree(node);
      spin_unlock(&trustedcell_response_lock);

      return permit;
    }
  }

	spin_unlock(&trustedcell_response_lock);

	return -ENODATA;
}

static int trustedcell_wait_for_response(int64_t request_id, bool *cachable)
{
  int status;
  int permit;

  status = wait_event_interruptible(trustedcell_response_wait_queue, 
      (permit = trustedcell_get_response(request_id, cachable)) >= 0);
  return (status < 0 ? status : permit);
}

int trustedcell_invoke_host(bool *cachable,
    kuid_t uid, struct trustedcell_id *cell,
    const char *category, const char *owner, const char *action,
    gfp_t gfp)
{
  int status;
  int64_t request_id = trustedcell_pull_request_id();
  category = kstrdup(category, gfp);
  if (!category) {
    goto out_free_category;
  }
  owner = kstrdup(owner, gfp);
  if (!owner) {
    goto out_free_owner;
  }
  action = kstrdup(action, gfp);
  if (!action) {
    goto out_free_action;
  }

  struct trustedcell_request request = {
    .request_id = request_id,
    .uid = uid,
    .cell = cell,
    .category = category,
    .owner = owner,
    .action = action,
  };
  struct response_table_node *resp;

  resp = trustedcell_register_response(request_id);
  if (!resp) {
    trustedcell_free_request(request);
    return -ENOMEM;
  }
  trustedcell_get_id(cell);
  status = trustedcell_send_request(request);
  if (status < 0) {
    goto out_unregister_response;
  }
  status = trustedcell_wait_for_response(request_id, cachable);
  if (status < 0) {
    goto out_unregister_response;
  }
  return (!status) ? -EACCES : 0;

out_unregister_response:
  trustedcell_unregister_response(resp);
  return status;

out_free_action:
  kfree(action);
out_free_owner:
  kfree(owner);
out_free_category:
  kfree(category);
  return -ENOMEM;
}

static int __init trustedcell_enforce_queue_init(void)
{
  hash_init(trustedcell_response_table);
  return 0;
}

core_initcall(trustedcell_enforce_queue_init);
