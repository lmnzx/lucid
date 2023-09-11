#include <assert.h>
#include <stdlib.h>

#include "hashtable.h"

// n must be a power of 2
static void h_init(HTab *htab, size_t n)
{
  assert(n > 0 && ((n - 1) & n) == 0);
  htab->tab = (HNode **)calloc(sizeof(HNode *), n);
  htab->mask = n - 1;
  htab->size = 0;
}

// hashtable insertion
static void h_insert(HTab *htab, HNode *node)
{
  size_t pos = node->hcode & htab->mask;
  HNode *next = htab->tab[pos];
  node->next = next;
  htab->tab[pos] = node;
  htab->size++;
}

// hashtable look up subroutine.
static HNode **h_lookup(
    HTab *htab, HNode *key, bool (*cmp)(HNode *, HNode *))
{
  if (!htab->tab)
  {
    return nullptr;
  }

  size_t pos = key->hcode & htab->mask;
  HNode **from = &htab->tab[pos];
  while (*from)
  {
    if (cmp(*from, key))
    {
      return from;
    }
    from = &(*from)->next;
  }
  return nullptr;
}

// remove a node from the chain
static HNode *h_detach(HTab *htab, HNode **from)
{
  HNode *node = *from;
  *from = node->next;
  // node->next = nullptr;
  htab->size--;
  return node;
}

const size_t k_resizing_work = 128; // 2^7

static void hm_help_resizing(HMap *hmap)
{
  if (hmap->ht2.tab == nullptr)
  {
    return;
  }

  size_t nwork = 0;
  while (nwork < k_resizing_work && hmap->ht2.size > 0)
  {
    // same for nodes from ht2 and move them to ht1
    HNode **from = &hmap->ht2.tab[hmap->resizing_pos];
    if (!*from)
    {
      hmap->resizing_pos++;
      continue;
    }

    h_insert(&hmap->ht1, h_detach(&hmap->ht2, from));
    nwork++;
  }

  if (hmap->ht2.size == 0)
  {
    // done
    free(hmap->ht2.tab);
    hmap->ht2.tab = nullptr;
  }
}

static void hm_start_resizing(HMap *hmap)
{
  assert(hmap->ht2.tab == nullptr);

  // create a bigger hashtable and swap them
  hmap->ht2 = hmap->ht1;
  h_init(&hmap->ht1, (hmap->ht1.mask + 1) * 2); // double the size
  hmap->resizing_pos = 0;
}

HNode *hm_lookup(
    HMap *hmap, HNode *key, bool (*cmp)(HNode *, HNode *))
{
  hm_help_resizing(hmap);
  HNode **from = h_lookup(&hmap->ht1, key, cmp);
  if (!from)
  {
    from = h_lookup(&hmap->ht2, key, cmp);
  }

  return from ? *from : nullptr;
}

const size_t k_max_load_factor = 8;

void hm_insert(HMap *hmap, HNode *node)
{
  if (!hmap->ht1.tab)
  {
    h_init(&hmap->ht1, 4);
  }

  h_insert(&hmap->ht1, node);

  if (!hmap->ht2.tab)
  {
    // check whether we need to resize
    size_t load_facotor = hmap->ht1.size / (hmap->ht1.mask + 1);
    if (load_facotor >= k_max_load_factor)
    {
      hm_start_resizing(hmap);
    }
  }
  hm_help_resizing(hmap);
}

HNode *hm_pop(
    HMap *hmap, HNode *key, bool (*cmp)(HNode *, HNode *))
{
  hm_help_resizing(hmap);

  HNode **from = h_lookup(&hmap->ht1, key, cmp); // look up from ht1
  if (from)
  {
    return h_detach(&hmap->ht1, from);
  }

  from = h_lookup(&hmap->ht2, key, cmp); // look up from ht2
  if (from)
  {
    return h_detach(&hmap->ht2, from);
  }

  return nullptr; // not found
}

size_t hm_size(HMap *hmap)
{
  return hmap->ht1.size + hmap->ht2.size;
}

void hm_destroy(HMap *hmap)
{
  assert(hmap->ht1.size + hmap->ht2.size == 0);
  free(hmap->ht1.tab);
  free(hmap->ht2.tab);
  *hmap = HMap();
}