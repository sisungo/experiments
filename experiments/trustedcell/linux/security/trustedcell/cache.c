/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * TrustedCell LSM - Enforce Caches
 *
 * Copyright (C) 2024 sisungo <sisungo@icloud.com>
 */

#include <linux/hashtable.h>
#include <linux/spinlock.h>
#include <linux/slab.h>

#include "include/cache.h"
#include "include/util.h"

struct enforce_cache_node {
  struct trustedcell_enforce_cache_item item;
  atomic_t popularity;
  struct hlist_node list;
  struct rcu_head rcu;
};

struct trustedcell_enforce_cache {
  struct hlist_head slots[TRUSTEDCELL_ENFORCE_CACHE_SLOTS];
  spinlock_t slot_locks[TRUSTEDCELL_ENFORCE_CACHE_SLOTS];
};

static struct trustedcell_enforce_cache enforce_cache;

static uint32_t enforce_cache_hashfn(
    struct trustedcell_enforce_cache_item item)
{
  return 0;
}

static bool enforce_cache_item_match(
    struct trustedcell_enforce_cache_item *pattern,
    struct trustedcell_enforce_cache_item *obj)
{
  bool p = pattern->uid == obj->uid
    && strcmp(pattern->cell->str, obj->cell->str) == 0
    && strcmp(pattern->category, obj->category) == 0
    && (pattern->category[0] == '~' ? strcmp(pattern->owner, obj->owner) == 0 : true)
    && strcmp(pattern->action, obj->action) == 0;
  return p;
}

static void enforce_cache_item_free(struct trustedcell_enforce_cache_item item)
{
  kfree(item.category);
  kfree(item.owner);
  kfree(item.action);
  trustedcell_put_id(item.cell);
}

static void enforce_cache_node_free(struct rcu_head *rcu)
{
  struct enforce_cache_node *node
    = container_of(rcu, struct enforce_cache_node, rcu);
  kfree(node);
}

void trustedcell_enforce_cache_init(void)
{
  for (int i = 0; i < TRUSTEDCELL_ENFORCE_CACHE_SLOTS; i += 1) {
    INIT_HLIST_HEAD(&enforce_cache.slots[i]);
    spin_lock_init(&enforce_cache.slot_locks[i]);
  }
}

int trustedcell_enforce_cache_add(struct trustedcell_enforce_cache_item item)
{
  struct enforce_cache_node *node;
  struct enforce_cache_node *i;
  uint32_t slot = enforce_cache_hashfn(item) % TRUSTEDCELL_ENFORCE_CACHE_SLOTS;
  int avg_popularity = 0;
  int nodes;

  node = kmalloc(sizeof(*node), GFP_KERNEL);
  node->item = item;
  if (!node) {
    return -ENOMEM;
  }
  spin_lock_bh(&enforce_cache.slot_locks[slot]);
  nodes = hlist_count_nodes(&enforce_cache.slots[slot]);
  if (nodes >= TRUSTEDCELL_ENFORCE_CACHE_SLOT_SIZE) {
    hlist_for_each_entry(i, &enforce_cache.slots[slot], list) {
      avg_popularity += atomic_read(&i->popularity);
    }
    avg_popularity /= nodes;
    hlist_for_each_entry(i, &enforce_cache.slots[slot], list) {
      if (atomic_read(&i->popularity) <= avg_popularity) {
        hlist_del_rcu(&i->list);
        enforce_cache_item_free(i->item);
        call_rcu(&i->rcu, enforce_cache_node_free);
      }
    }
  }
  hlist_for_each_entry(i, &enforce_cache.slots[slot], list) {
    if (enforce_cache_item_match(&item, &i->item)) {
      kfree(node);
      enforce_cache_item_free(i->item);
      i->item = item;
      spin_unlock_bh(&enforce_cache.slot_locks[slot]);
      return 0;
    }
  }
  atomic_set(&node->popularity, 1);
  hlist_add_head_rcu(&node->list, &enforce_cache.slots[slot]);
  spin_unlock_bh(&enforce_cache.slot_locks[slot]);

  return 0;
}

int trustedcell_enforce_cache_match(struct trustedcell_enforce_cache_item mat)
{
  uint32_t slot = enforce_cache_hashfn(mat) % TRUSTEDCELL_ENFORCE_CACHE_SLOTS;
  struct enforce_cache_node *i;
  rcu_read_lock();
  hlist_for_each_entry_rcu(i, &enforce_cache.slots[slot], list) {
    if (enforce_cache_item_match(&mat, &i->item)) {
      rcu_read_unlock();
      atomic_inc(&i->popularity);
      return i->item.resp;
    }
  }
  rcu_read_unlock();
  return -ENODATA;
}
