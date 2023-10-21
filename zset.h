#pragma once

#include "avl.h"
#include "hashtable.h"

struct ZSet {
    AVLNode *tree = nullptr;
    HMap hmap;
};

struct ZNode {
    AVLNode tree;
    HNode hmap;
    double score;
    size_t len;
    char name[0];
};

bool zset_add(ZSet *zset, const char *name, size_t len, double score);

ZNode *zset_lookup(ZSet *zset, const char *name, size_t len);

ZNode *zset_pop(ZSet *zset, const char *name, size_t len);

ZNode *zset_query(ZSet *zset, double score, const char *name, size_t len, int64_t offset);

void zset_dispose(ZSet *zset);

void znode_del(ZNode *node);