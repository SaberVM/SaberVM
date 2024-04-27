/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// This is the runtime system for SaberVM on a 64-bit little-endian architecture.

#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

typedef uint64_t u64;
typedef int64_t i64;
typedef uint32_t u32;
typedef uint8_t u8;
typedef int32_t i32;

/*
 * A pointer to an object within a region.
 * The `generation` field is used to detect when a pointer becomes invalid.
 * The `reference` field is the actual pointer.
 */
typedef struct {
    i64 generation;
    u8 *reference;
} Pointer;

/*
 * A region (growable, nonmoving arena) of memory.
 * The type system ensures pointers into the region aren't dereferenced after the region is freed.
 * In the future I'll likely switch to a non-growing region where the size is given.
 */
typedef struct {
    size_t offset;
    size_t capacity;
    u8 data[];
} Region;

typedef u8 Stack[4096];

/*
 * Allocate a new region.
 * The type system ensures memory is written to before it is read,
 * so there's no need to initialize the memory.
 */
Region *new_region();

/*
 * Allocate an object in a region 
 * The type system ensures it gets initialized before it is read,
 * so there's no need to initialize the memory.
 */
Pointer alloc_object(Region *r, u64 size);

/*
 * Crash if the given pointer is no longer valid.
 * This happens if the object it's pointing at has been freed.
 * In the future this will jump to the exception handler instead of crashing.
 */
void check_ptr(Pointer ptr);

/*
 * Free an object within a region. 
 * Generations are used to keep this safe, instead of static analysis.
 */
void free_object(Pointer ptr);

/*
 * Free a region of memory.
 * Static analysis is used to keep this safe, instead of generations.
 */
void free_region(Region *r);

/*
 * The actual VM implementation.
 */
extern uint8_t vm_function(u8 instrs[], size_t instrs_len);
