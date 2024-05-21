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

#define METADATA_OFFSET (sizeof(u64) + sizeof(u64))

Region *new_region(size_t size) {
    Region *r = malloc(sizeof(size_t) + sizeof(size_t) + sizeof(size_t) + size); // an extra sizeof(size_t) to be safe re: padding
    r->offset = 0;
    r->capacity = size;
    return r;
}

Pointer alloc_object(Region *r, u64 size) {
    // I'd love to figure out how to have less conditionals in this function, but it's just a prototype.
    if (r->offset + METADATA_OFFSET + size > r->capacity) {
        for (size_t offset = 0; offset < r->offset; offset += METADATA_OFFSET + r->data[offset]) {
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
        }
        dbg("r->offet: %lu, size: %lu, r->capacity: %lu\n", r->offset, size, r->capacity);
        printf("Runtime Error! Allocation too big for region!\n");
        exit(1); // this will jump to an exception handler eventually
    } else {
        i64 first_generation = 1;
        memcpy(r->data + r->offset, &first_generation, sizeof(first_generation));
        memcpy(r->data + r->offset + sizeof(first_generation), &size, sizeof(size));
        Pointer ptr = {first_generation, &(r->data[r->offset]) + METADATA_OFFSET};
        dbg("alloc object: gen: %ld, size: %lu\n", ptr.generation, size);
        r->offset += METADATA_OFFSET + size;
        for (size_t i = 0; i < r->offset; i++) {
            dbg(" %d", r->data[i]);
        }
        dbg("\n");
        return ptr;
    }
}

void check_ptr(Pointer ptr) {
    dbg("check ptr:\n");
    for (int i = 0; i < 20; i++) {
        dbg(" %d",  *(u8*)(ptr.reference - METADATA_OFFSET - 16 + i));
    }
    dbg("\n");
    if (ptr.generation < 0) {
        // negative generation in a pointer means the referent is unfreeable
        // and therefore doesn't have a generation tag in the preceding memory
        return;
    }
    i64 g;
    memcpy(&g, ptr.reference - METADATA_OFFSET, sizeof(g));
    dbg("check generation %ld\n", g);
    if (ptr.generation != g) {
        dbg("%ld != %ld\n", ptr.generation, g);
        printf("Runtime Error! The program is trying to access memory that's already been freed!\n");
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

#define INSTR_PARAM(t, name) \
    t name; \
    memcpy(&name, instrs + pc, sizeof(name)); \
    pc += sizeof(name); \

#define POP(t, name) \
    t name; \
    if (sp == 0 && stack->last != NULL) { stack = stack->last; sp = stack->saved_sp; } \
    sp -= sizeof(name); \
    memcpy(&name, stack->data + sp, sizeof(name));

// push a value onto the stack.
// no `ensure_size` here because the caller will often know that it's not necessary.
#define PUSH(t, e) \
    {t x = e; \
    memcpy(stack->data + sp, &x, sizeof(x)); \
    sp += sizeof(x);}

// start a new contiguous stack chunk if the given size wouldn't fit.
// The caller must guarantee that the given size is less than STACK_CHUNK_SIZE
void ensure_size(struct Stack *stack, u32 *sp, size_t size) {
    if (*sp + size > STACK_CHUNK_SIZE) {
        struct Stack *new_stack = malloc(sizeof(struct Stack));
        new_stack->last = stack;
        new_stack->saved_sp = *sp;
        *sp = 0;
        stack = new_stack;
    }
}

uint8_t vm_function(u8 instrs[], size_t instrs_len) {
    for (u32 i = 0; i < instrs_len; i++) {
        dbg(" %d", instrs[i]);
    }
    dbg("\n");
    u32 data_section_size;
    memcpy(&data_section_size, instrs, sizeof(data_section_size));
    u32 pc = sizeof(data_section_size) + data_section_size;
    u32 sp = 0;
    struct Stack *stack = malloc(sizeof(struct Stack));
    while (1) {
        dbg("pc: %d, sp: %d\n", pc, sp);
        for (u32 i = 0; i < sp; i++) {
            dbg(" %d", stack->data[i]);
        }
        dbg("\n");
        switch (instrs[pc]) {
        case 0: {
            dbg("get!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            INSTR_PARAM(size_t, size);
            ensure_size(stack, &sp, size);
            memcpy(stack->data + sp, stack->data + sp - offset - size, size);
            sp += size;
            break;
        }
        case 1: {
            dbg("init!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            INSTR_PARAM(size_t, size);
            INSTR_PARAM(size_t, tpl_size);
            sp -= size;
            memcpy(stack->data + sp - tpl_size + offset, stack->data + sp, size);
            break;
        }
        case 2: {
            dbg("init in-place!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            INSTR_PARAM(size_t, size);
            Pointer ptr; 
            sp -= size + sizeof(ptr);
            memcpy(&ptr, stack->data + sp, sizeof(ptr));
            check_ptr(ptr);
            memcpy(ptr.reference + offset, stack->data + sp + sizeof(ptr), size);
            PUSH(Pointer, ptr);
            break;
        }
        case 3: {
            dbg("malloc!\n");
            pc++;
            INSTR_PARAM(size_t, size);
            POP(Region*, handle);
            ensure_size(stack, &sp, sizeof(handle));
            PUSH(Pointer, alloc_object(handle, size));
            break;
        }
        case 4: {
            dbg("alloca!\n");
            pc++;
            INSTR_PARAM(size_t, size);
            ensure_size(stack, &sp, size);
            sp += size;
            break;
        }
        case 5: {
            dbg("projection!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            INSTR_PARAM(size_t, size);
            INSTR_PARAM(size_t, tpl_size);
            sp -= tpl_size;
            memcpy(stack->data + sp, stack->data + sp + offset, size);
            sp += size;
            break;
        }
        case 6: {
            dbg("projection in-place!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            INSTR_PARAM(size_t, size);
            POP(Pointer, ptr);
            check_ptr(ptr);
            ensure_size(stack, &sp, size);
            memcpy(stack->data + sp, ptr.reference + offset, size);
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
            POP(Pointer, ptr);
            if (ptr.generation == -1) {
                // -1 generation means data section string
                size_t size = (size_t)instrs + 4 + (size_t)data_section_size - (size_t)ptr.reference;
                printf("%.*s", (int)size, ptr.reference);
            } else {
                check_ptr(ptr);
                size_t array_len;
                memcpy(&array_len, ptr.reference, sizeof(array_len));
                printf("%.*s", (int)array_len, ptr.reference + sizeof(array_len));
            }
            break;
        }
        case 9: {
            dbg("literal!\n");
            pc++;
            INSTR_PARAM(i32, lit);
            ensure_size(stack, &sp, sizeof(lit));
            PUSH(i32, lit);
            break;
        }
        case 10: {
            dbg("global function!\n");
            pc++;
            INSTR_PARAM(u32, lit);
            ensure_size(stack, &sp, sizeof(lit));
            PUSH(u32, lit);
            break;
        }
        case 11: {
            dbg("halt!\n");
            POP(u8, status_code);
            exit(status_code);
            break;
        }
        case 12: {
            dbg("new region!\n");
            pc++;
            INSTR_PARAM(size_t, size);
            Region *r = new_region(size);
            ensure_size(stack, &sp, sizeof(r));
            PUSH(Region*, r);
            break;
        }
        case 13: {
            dbg("free region!\n");
            pc++;
            POP(Region*, r);
            free(r);
            break;
        }
        case 14: {
            dbg("dereference pointer!\n");
            pc++;
            INSTR_PARAM(size_t, size);
            POP(Pointer, ptr);
            check_ptr(ptr);
            ensure_size(stack, &sp, size);
            memcpy(stack->data + sp, ptr.reference, size);
            sp += size;
            break;
        }
        case 15: {
            dbg("new array!\n");
            pc++;
            INSTR_PARAM(size_t, elem_size);
            POP(i32, len);
            POP(Region*, r);
            size_t size = elem_size * len;
            dbg("size: %ld\n", sizeof(size) + size);
            Pointer ptr = alloc_object(r, sizeof(size) + size);
            memcpy(ptr.reference, &size, sizeof(size));
            memset(ptr.reference + sizeof(size), 0, size);
            ensure_size(stack, &sp, sizeof(ptr));
            PUSH(Pointer, ptr);
            break;
        }
        case 16: {
            dbg("mutate array component!\n");
            pc++;
            INSTR_PARAM(size_t, elem_size);
            POP(i32, i);
            Pointer ptr;
            memcpy(&ptr, stack->data + sp - elem_size - sizeof(ptr), sizeof(ptr));
            size_t n = elem_size * i;
            size_t array_len;
            memcpy(&array_len, ptr.reference, sizeof(array_len));
            if (n + elem_size > array_len) {
                printf("Runtime Error! Array index out of bounds during an initialization.\n");
                exit(1);
            }
            memcpy(ptr.reference + sizeof(array_len) + n, stack->data + sp - elem_size, elem_size);
            sp -= elem_size + sizeof(ptr);
            PUSH(Pointer, ptr);
            break;
        }
        case 17: {
            dbg("project from array!\n");
            pc++;
            INSTR_PARAM(size_t, elem_size);
            POP(i32, i);
            size_t n = elem_size * i;
            POP(Pointer, ptr);
            check_ptr(ptr);
            size_t array_len;
            memcpy(&array_len, ptr.reference, sizeof(array_len));
            if (n + elem_size > array_len) {
                printf("Runtime Error! Array index out of bounds during a projection.\n");
                exit(1);
            }
            ensure_size(stack, &sp, elem_size);
            memcpy(stack->data + sp, ptr.reference + sizeof(array_len) + n, elem_size);
            sp += elem_size;
            break;
        }
        case 18: {
            dbg("add two i32s!\n");
            pc++;
            POP(i32, a);
            POP(i32, b);
            PUSH(i32, a + b);
            break;
        }
        case 19: {
            dbg("multiply two i32s!\n");
            pc++;
            POP(i32, a);
            POP(i32, b);
            PUSH(i32, a * b);
            break;
        }
        case 20: {
            dbg("divide two i32s!\n");
            pc++;
            POP(i32, a);
            POP(i32, b);
            PUSH(i32, a / b);
            break;
        }
        case 21: {
            dbg("call if not zero!\n");
            POP(u32, f);
            POP(u32, g);
            POP(i32, cond);
            dbg("%d\n", cond);
            if (cond != 0) {
                pc = g;
            } else {
                pc = f;
            }
            break;
        }
        case 22: {
            dbg("load from data section!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            Pointer ptr = (Pointer){
                .reference = instrs + 4 + offset, 
                // negative generation in a pointer means the referent is unfreeable. In this case, the referent is in the data section.
                .generation = -1 
            };
            PUSH(Pointer, ptr);
            break;
        }
        case 23: {
            dbg("project from data-section array!\n");
            pc++;
            INSTR_PARAM(size_t, elem_size);
            POP(i32, i);
            size_t n = elem_size * i;
            POP(Pointer, ptr); // frontend ensures this is a data-section pointer, so we don't need to check it.
            if (n + elem_size > data_section_size) {
                printf("Runtime Error! Array index out of bounds during a projection from the data section.\n");
                exit(1);
            }
            ensure_size(stack, &sp, elem_size);
            memcpy(stack->data + sp, ptr.reference + n, elem_size);
            sp += elem_size;
            break;
        }
        case 24: {
            dbg("print n characters!\n");
            pc++;
            POP(i32, n);
            POP(Pointer, ptr);
            if (ptr.generation == -1) {
                // -1 generation means data section string
                size_t rest_of_data_section = instrs + 4 + data_section_size - ptr.reference;
                u32 size = (u32)n;
                if (size > rest_of_data_section) {
                    size = rest_of_data_section;
                }
                printf("%.*s", (int)size, ptr.reference);
            } else {
                check_ptr(ptr);
                size_t array_len;
                memcpy(&array_len, ptr.reference, sizeof(array_len));
                u32 size = (u32)n;
                if (size > array_len) {
                    size = array_len;
                }
                printf("%.*s", (int)size, ptr.reference + sizeof(array_len));
            }
            break;
        }
        case 25: {
            dbg("u8 literal!\n");
            pc++;
            INSTR_PARAM(u8, val);
            ensure_size(stack, &sp, sizeof(val));
            PUSH(u8, val);
            break;
        }
        case 26: {
            dbg("add u8!\n");
            pc++;
            POP(u8, a);
            POP(u8, b);
            PUSH(u8, a + b);
            break;
        }
        case 27: {
            dbg("multiply u8!\n");
            pc++;
            POP(u8, a);
            POP(u8, b);
            PUSH(u8, a * b);
            break;
        }
        case 28: {
            dbg("divide u8!\n");
            pc++;
            POP(u8, a);
            POP(u8, b);
            PUSH(u8, a / b);
            break;
        }
        default: {
            printf("internal error!! Unknown IR op %d, please let the SaberVM team know!!", instrs[pc]);
            exit(1);
        }
        }
    }
}
