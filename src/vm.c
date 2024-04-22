/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// This is the runtime system for SaberVM on a 64-bit little-endian architecture.

#include "vm.h"

#define DEBUG 0
#if DEBUG
#define dbg(...) printf(__VA_ARGS__)
#else
#define dbg(...)
#endif

#define METADATA_OFFSET sizeof(u64) + sizeof(u64)

Region *new_region() {
    Region *r = malloc(sizeof(Region));
    r->offset = 0;
    r->next = NULL;
    return r;
}

Pointer alloc_object(Region *r, u64 size) {
    // I'd love to figure out how to have less conditionals in this function, but it's just a prototype.
    size_t offset = 0;
    while (offset < r->offset) {
        // negative generation means free
        // the absolute value of the generation is what the last generation was, then we add one to get the current generation
        i64 local_generation;
        memcpy(&local_generation, r->data + offset, sizeof(local_generation));
        u64 local_size;
        memcpy(&local_size, r->data + offset + sizeof(local_generation), sizeof(local_size));
        if (local_generation < 0 /*freed*/ && local_size <= size /*fits*/) {
            i64 new_generation = -local_generation + 1;
            memcpy(r->data + offset, &new_generation, sizeof(new_generation));
            memcpy(r->data + offset + sizeof(new_generation), &size, sizeof(size));
            r->offset = offset + METADATA_OFFSET + size;
            return (Pointer){new_generation, r->data + offset + METADATA_OFFSET}; // pointer skips over the generation and size
        }
        offset += METADATA_OFFSET + r->data[offset];
    }
    if (r->offset + METADATA_OFFSET + size > 4096) {
        if (r->next == NULL) {
            r->next = new_region();
        }
        return alloc_object(r->next, size);
    }
    else {
        i64 first_generation = 1;
        memcpy(r->data + r->offset, &first_generation, sizeof(first_generation));
        memcpy(r->data + r->offset + sizeof(first_generation), &size, sizeof(size));
        Pointer ptr = {first_generation, &(r->data[r->offset])};
        r->offset += METADATA_OFFSET + size;
        dbg("ptr: %llu\n", (u64)(r->data));
        dbg("ptr: %lld, %llu.\n", ptr.generation, (u64)(ptr.reference));
        return ptr;
    }
}

void check_ptr(Pointer ptr) {
    i64 g;
    memcpy(&g, &ptr.reference - METADATA_OFFSET, sizeof(g));
    if (ptr.generation != g) {
        dbg("%lld != %lld\n", ptr.generation, g);
        printf("Runtime error! The program is trying to access memory that's already been freed!\n");
        exit(1); // this will be a jump to exception handler soon
    }
}

void free_object(Pointer ptr) {
    check_ptr(ptr);
    i64 g;
    memcpy(&g, ptr.reference - METADATA_OFFSET, sizeof(g));
    g = -g;
    memcpy(ptr.reference - METADATA_OFFSET, &g, sizeof(g));
}

void free_region(Region *r) {
    if (r->next != NULL) {
        free_region(r->next);
    }
    free(r);
}

size_t instruction_param(u8 instrs[], u32 *pc) {
    size_t value;
    memcpy(&value, instrs + *pc, sizeof(value));
    *pc += sizeof(value);
    return value;
}

#define POP(t, name) \
    t name; \
    memcpy(&name, stack + sp, sizeof(name)); \
    sp -= sizeof(name);

#define PUSH(t, e) \
        {t x = e; \
        memcpy(stack + sp, &x, sizeof(x)); \
        sp += sizeof(x);}

uint8_t vm_function(u8 instrs[], size_t instrs_len) {
    for (u32 i = 0; i < instrs_len; i++) {
        dbg(" %d", instrs[i]);
    }
    dbg("\n");
    u32 pc = 0;
    u8 sp = 0;
    Stack stack;
    while (1) {
        dbg("pc: %d\n", pc);
        switch (instrs[pc]) {
        case 0: {
            dbg("get!\n");
            pc++;
            size_t offset = instruction_param(instrs, &pc);
            size_t size = instruction_param(instrs, &pc);
            memcpy(stack + sp, stack + sp - offset - size, size);
            sp += size;
            break;
        }
        case 1: {
            dbg("init!\n");
            pc++;
            size_t offset = instruction_param(instrs, &pc);
            size_t size = instruction_param(instrs, &pc);
            size_t tpl_size = instruction_param(instrs, &pc);
            sp -= size;
            memcpy(stack + sp - tpl_size + offset, stack + sp, size);
            break;
        }
        case 2: {
            dbg("init in-place!\n");
            pc++;
            size_t offset = instruction_param(instrs, &pc);
            size_t size = instruction_param(instrs, &pc);
            Pointer ptr; 
            sp -= size + sizeof(ptr);
            memcpy(&ptr, stack + sp, sizeof(ptr));
            check_ptr(ptr);
            memcpy(ptr.reference + offset, stack + sp + sizeof(ptr), size);
            PUSH(Pointer, ptr);
            break;
        }
        case 3: {
            dbg("malloc!\n");
            pc++;
            size_t size = instruction_param(instrs, &pc);
            POP(Region*, handle);
            PUSH(Pointer, alloc_object(handle, size));
            break;
        }
        case 4: {
            dbg("alloca!\n");
            pc++;
            size_t size = instruction_param(instrs, &pc);
            sp += size;
            break;
        }
        case 5: {
            dbg("projection!\n");
            pc++;
            size_t offset = instruction_param(instrs, &pc);
            size_t size = instruction_param(instrs, &pc);
            size_t tpl_size = instruction_param(instrs, &pc);
            sp -= tpl_size;
            memcpy(stack + sp, stack + sp + offset, size);
            sp += size;
            break;
        }
        case 6: {
            dbg("projection in-place!\n");
            pc++;
            size_t offset = instruction_param(instrs, &pc);
            size_t size = instruction_param(instrs, &pc);
            POP(Pointer, ptr);
            check_ptr(ptr);
            memcpy(stack + sp, ptr.reference + offset, size);
            sp += size;
            break;
        }
        case 7: {
            dbg("call!\n");
            POP(u32, new_pc);
            pc = new_pc;
            break;
        }
        case 8: {
            dbg("print!\n");
            pc++;
            POP(i64, value);
            printf("%ld\n", value);
            break;
        }
        case 9: {
            dbg("literal!\n");
            pc++;
            i32 lit;
            memcpy(&lit, instrs + pc, sizeof(lit));
            pc += sizeof(lit);
            PUSH(i32, lit);
            break;
        }
        case 10: {
            dbg("global function!\n");
            pc++;
            size_t lit = instruction_param(instrs, &pc);
            PUSH(size_t, lit);
            break;
        }
        case 11: {
            dbg("halt!\n");
            POP(i32, status_code);
            exit(status_code);
            break;
        }
        case 12: {
            dbg("new region!\n");
            pc++;
            Region *r = new_region();
            PUSH(Region*, r);
            break;
        }
        case 13: {
            dbg("free region!\n");
            pc++;
            POP(Region*, r);
            free_region(r);
            break;
        }
        case 14: {
            dbg("dereference pointer!\n");
            pc++;
            size_t size = instruction_param(instrs, &pc);
            POP(Pointer, ptr);
            check_ptr(ptr);
            memcpy(stack + sp, ptr.reference, size);
            sp += size;
            break;
        }
        }
    }
}