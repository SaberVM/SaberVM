/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#define DEBUG 0
#if DEBUG
#define dbg(...) printf(__VA_ARGS__)
#else
#define dbg(...)
#endif

typedef uint64_t u64;
typedef int64_t i64;
typedef uint32_t u32;
typedef struct {
    i64 generation;
    u32 *reference;
} Pointer;

#define expected_generation(ptr) ptr.generation
#define allocation_generation(ptr) (u32*)(ptr.reference - 4)
#define allocation_size(ptr) (u32*)(ptr.reference - 2)
#define allocation_data(ptr) ptr.reference
#define new_ptr(generation, reference) (Pointer){generation, reference}

struct Region {
    size_t offset;
    struct Region *next;
    u32 data[1024];
};

struct Region *new_region() {
    struct Region *r = malloc(sizeof(struct Region));
    r->offset = 0;
    r->next = NULL;
    return r;
}

Pointer alloc_object(struct Region *r, u64 size) {
    // I'd love to figure out how to have less conditionals in this function, but it's a fast prototype.
    size_t offset = 0;
    while (offset < r->offset) {
        // negative generation means free
        // the absolute value of the generation is what the last generation was, so we add one to get the current generation
        u64 local_generation = (u64)r->data[offset] << 32 | (u64)r->data[offset+1];
        u64 local_size = (u64)r->data[offset+2] << 32 | (u64)r->data[offset+3];
        if (local_generation < 0 /*freed*/ && local_size <= size /*fits*/) {
            i64 new_generation = -local_generation + 1;
            r->data[offset++] = (u32)(new_generation >> 32);
            r->data[offset++] = (u32)(new_generation & 0xFFFFFFFF);
            r->data[offset++] = (u32)(size >> 32);
            r->data[offset++] = (u32)(size & 0xFFFFFFFF);
            r->offset = offset + size;
            return new_ptr(new_generation, &(r->data[offset])); // pointer skips over the generation and size
        }
        offset += 4 + r->data[offset];
    }
    if (r->offset + 4 + size > 1024) {
        if (r->next == NULL) {
            r->next = new_region();
        }
        return alloc_object(r->next, size);
    } else {
        r->data[r->offset++] = 0;
        r->data[r->offset++] = 1; // first generation
        r->data[r->offset++] = (u32)(size >> 32);
        r->data[r->offset++] = (u32)(size & 0xFFFFFFFF);
        Pointer ptr = new_ptr(1/*first generation*/, &(r->data[r->offset]));
        r->offset += size;
        dbg("ptr: %llu\n", (u64)(r->data));
        dbg("ptr: %lld, %llu.\n", ptr.generation, (u64)(ptr.reference));
        return ptr;
    }
}

void check_ptr(Pointer ptr);

void free_object(Pointer ptr) {
    check_ptr(ptr);
    u32 *alloc_gen = allocation_generation(ptr);
    i64 g = (i64)(*alloc_gen) << 32 | (i64)(*(alloc_gen+1));
    g = -g;
    *alloc_gen = (u32)(g >> 32);
    *(alloc_gen+1) = (u32)(g & 0xFFFFFFFF);
}

void check_ptr(Pointer ptr) {
    u32 *alloc_gen = allocation_generation(ptr);
    i64 g = (i64)(*alloc_gen) << 32 | (i64)(*(alloc_gen+1));
    i64 expected_gen = expected_generation(ptr);
    if (expected_gen != g) {
        dbg("%lld != %lld\n", expected_gen, g);
        printf("Runtime error! The program is trying to access memory that's already been freed!\n");
        exit(1); // this will be a jump to exception handler soon
    }
}

void free_region(struct Region *r) {
    if (r->next != NULL) {
        free_region(r->next);
    }
    free(r);
}

#define get_64_bit ((u64)(stack[sp-2]) << 32 | (u64)(stack[sp-1]))

void push(u32 stack[1024], u32 *sp, u32 *value, size_t size) {
    switch (size) {
        case 1:
            stack[(*sp)++] = *value;
            break;
        case 2:
            stack[(*sp)++] = *(value++);
            stack[(*sp)++] = *value;
            break;
        case 4:
            stack[(*sp)++] = *(value++);
            stack[(*sp)++] = *(value++);
            stack[(*sp)++] = *(value++);
            stack[(*sp)++] = *value;
            break;
        default:
            printf("Internal error! Unsupported push size! Please reach out to SaberVM staff!\n");
            exit(1);
    }
}

void push_val_64(u32 stack[1024], u32 *sp, u64 value) {
    stack[(*sp)++] = (u32)(value >> 32);
    stack[(*sp)++] = (u32)(value & 0xFFFFFFFF);
}

void handle_init(u32 stack[], u32 *sp, size_t offset, size_t size);

extern uint8_t vm_function(u32 instrs[], size_t instrs_len);

uint8_t vm_function(u32 instrs[], size_t instrs_len) {
    for (u32 i = 0; i < instrs_len; i++) {
        dbg(" %d", instrs[i]);
    }
    dbg("\n");
    u32 pc = 0;
    u32 sp = 0;
    u32 stack[1024];
    while (1) {
        dbg("pc: %d\n", pc);
        switch (instrs[pc]) {
            case 0: // get 
                {
                    dbg("get!\n");
                    u32 offset = instrs[pc + 1];
                    u32 size = instrs[pc + 2];
                    push(stack, &sp, stack+sp-offset-size, size);
                    pc += 3;
                    break;
                }
            case 1: // init
                {
                    dbg("init!\n");
                    u32 offset = instrs[pc + 1];
                    u32 size = instrs[pc + 2];
                    handle_init(stack, &sp, (size_t)offset, (size_t)size);
                    pc += 3;
                }
                break;
            case 2: // malloc
                {
                    dbg("malloc!\n");
                    u32 size = instrs[pc + 1];
                    struct Region *handle = (struct Region*)get_64_bit;
                    Pointer ptr = alloc_object(handle, size);
                    sp -= 2;
                    push_val_64(stack, &sp, ptr.generation);
                    push_val_64(stack, &sp, (u64)(ptr.reference));
                    pc += 2;
                }
                break;
            case 3: // projection
                {
                    dbg("projection!\n");
                    u32 offset = instrs[pc + 1];
                    u32 size = instrs[pc + 2];
                    u64 reference = get_64_bit;
                    sp -= 2;
                    u64 generation = get_64_bit;
                    sp -= 2;
                    Pointer ptr = {generation, (u32*)reference};
                    check_ptr(ptr);
                    u32 *data = allocation_data(ptr);
                    push(stack, &sp, data + offset, size);
                    pc += 3;
                }
                break;
            case 4: // call
                dbg("call!\n");
                pc = (u32)(stack[sp-1]);
                sp -= 1;
                break;
            case 5: // print
                dbg("print!\n");
                printf("%d\n", stack[sp-1]);
                sp -= 1;
                pc += 1;
                break;
            case 6: // literal 
                {
                    dbg("literal!\n");
                    u32 lit = instrs[pc + 1];
                    stack[sp] = lit;
                    sp += 1;
                    pc += 2;
                }
                break;
            case 7: // global function
                {
                    dbg("global function!\n");
                    unsigned int lit = instrs[pc + 1];
                    stack[sp] = lit;
                    sp += 1;
                    pc += 2;
                }
                break;
            case 8: // halt
                dbg("halt!\n");
                exit(instrs[pc + 1]);
                break;
            case 9: // new region
                {
                    dbg("new region!\n");
                    u64 r = (u64)new_region();
                    push_val_64(stack, &sp, r);
                    pc += 1;
                }
                break;
            case 10: // free region
                {
                    dbg("free region!\n");
                    u64 r = get_64_bit;
                    sp -= 2;
                    free_region((struct Region *)r);
                    pc += 1;
                }
        }
    }
}

#define get_64_bit_through_sp_ref ((u64)(stack[(*sp)-2]) << 32 | (u64)(stack[(*sp)-1]))

void handle_init(u32 stack[], u32 *sp, size_t offset, size_t size) {
    dbg("handling init!\n");
    switch (size) {
        case 1:
            {
                u32 val = stack[(*sp) - 1];
                *sp -= 1;
                u64 reference = get_64_bit_through_sp_ref;
                *sp -= 2;
                u64 generation = get_64_bit_through_sp_ref;
                *sp -= 2;
                Pointer ptr = {generation, (u32*)reference};
                dbg("ptr %lld %llu\n", ptr.generation, (u64)(ptr.reference));
                check_ptr(ptr);
                dbg("check successful!\n");
                *(u32*)(allocation_data(ptr) + offset) = val;
                dbg("set successful!\n");
                push_val_64(stack, sp, ptr.generation);
                push_val_64(stack, sp, (u64)(ptr.reference));
                dbg("push successful!\n");
            }
            break;
        case 2:
            {
                dbg("hi?\n");
                u64 val = get_64_bit_through_sp_ref;
                *sp -= 2;
                u64 reference = get_64_bit_through_sp_ref;
                *sp -= 2;
                u64 generation = get_64_bit_through_sp_ref;
                *sp -= 2;
                Pointer ptr = {generation, (u32*)reference};
                check_ptr(ptr);
                memcpy(allocation_data(ptr) + offset, &val, 8);
                push_val_64(stack, sp, ptr.generation);
                push_val_64(stack, sp, (u64)(ptr.reference));
            }
            break;
        case 4:
            {
                dbg("hello?\n");
                u64 val1 = get_64_bit_through_sp_ref;
                *sp -= 2;
                u64 val2 = get_64_bit_through_sp_ref;
                struct{u64 val1; u64 val2;} val = {val1, val2};
                *sp -= 2;
                u64 reference = get_64_bit_through_sp_ref;
                *sp -= 2;
                u64 generation = get_64_bit_through_sp_ref;
                *sp -= 2;
                Pointer ptr = {generation, (u32*)reference};
                check_ptr(ptr);
                memcpy(allocation_data(ptr) + offset, &val, 16);
                push_val_64(stack, sp, ptr.generation);
                push_val_64(stack, sp, (u64)(ptr.reference));
            }
            break;
        default:
            printf("Internal error! Unsupported init size! Please reach out to SaberVM staff!\n");
            exit(1);
    }
    dbg("handled init!\n");
}