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
    dbg("region size with metadata: %lu\n", sizeof(size_t) + sizeof(size_t) + sizeof(size_t) + size);
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
void ensure_size(struct Stack **stack, u32 *sp, size_t size) {
    if (*sp + size > STACK_CHUNK_SIZE) {
        struct Stack *new_stack = malloc(sizeof(struct Stack));
        new_stack->last = *stack;
        dbg("NEW STACK %u %lu\n", *sp, size);
        new_stack->saved_sp = *sp;
        *sp = 0;
        *stack = new_stack;
        dbg("%u\n", (*stack)->saved_sp);
    }
}

Handler scheduler[255];
u8 scheduler_len = 0;

int post_task(Handler h) {
    if (scheduler_len == 255) return 0;
    scheduler[scheduler_len++] = h;
    return 1;
}

u8 waiting = 0;
Handler stdin_handler = {0};
Region *stdin_rgn = NULL;
Pointer stdin_str_ptr = {0, NULL};
Handler stdout_handler = {0};
Handler stderr_handler = {0};

void handle_stdin() {
    ssize_t bytes;
    char buffer[1024];
    // Read all available input
    while ((bytes = read(STDIN_FILENO, buffer, sizeof(buffer))) > 0) {
        Pointer ptr = alloc_object(stdin_rgn, bytes + sizeof(bytes));
        memcpy(ptr.reference, &bytes, sizeof(bytes));
        memcpy(ptr.reference + sizeof(bytes), buffer, bytes);
        Handler h;
        memcpy(&h, &stdin_handler, sizeof(h));
        memcpy(h.param, &ptr, sizeof(ptr));
        h.param_size = sizeof(ptr);
        if (post_task(h)) {
            printf("failed to post stdin handler to scheduler\n");
            exit(1);
        }
        waiting &= 0b11111110;
    }
    
}

u8 vm_function(u8 instrs[]) {
    // for (u32 i = 0; i < instrs_len; i++) {
    //     dbg(" %d", instrs[i]);
    // }
    // dbg("\n");
    u32 data_section_size;
    memcpy(&data_section_size, instrs, sizeof(data_section_size));
    dbg("data section size: %lu\n", data_section_size);
    u32 pc = sizeof(data_section_size) + data_section_size;
    dbg("pc: %lu\n", pc);
    u32 sp = 0;
    struct Stack *stack = malloc(sizeof(struct Stack));

    int flags = fcntl(STDIN_FILENO, F_GETFL, 0);
    fcntl(STDIN_FILENO, F_SETFL, flags | O_NONBLOCK | O_ASYNC);
    fcntl(STDIN_FILENO, __F_SETOWN, getpid());
    signal(SIGIO, handle_stdin);

    Handler on_start = (Handler){.f=pc};
    post_task(on_start); // guaranteed to succeed; no failure check here
    while (1) {
        while (scheduler_len > 0) {
            Handler h = scheduler[--scheduler_len];
            memcpy(stack->data + sp, &h.param, h.param_size);
            sp += h.param_size;
            memcpy(stack->data + sp, &h.env, sizeof(h.env));
            sp += sizeof(h.env);
            u8 err = eval(instrs, h.f, sp + h.param_size + sizeof(h.env), data_section_size, stack);
            if (err) return err;
        }
        dbg("waiting: %d\nscheduler_len: %d\n", waiting);
        while (scheduler_len == 0 && waiting) usleep(10000);
        if (!waiting && scheduler_len == 0) {
            return 0;
        }
    }
}

u8 eval(u8 instrs[], u32 pc, u32 sp, u32 data_section_size, struct Stack *stack) {
    while (1) {
        // dbg("pc: %d, sp: %d\n", pc, sp);
        // for (u32 i = 0; i < sp; i++) {
        //     dbg(" %d", stack->data[i]);
        // }
        // dbg("\n");
        switch (instrs[pc]) {
        case 0: {
            dbg("get!\n");
            pc++;
            INSTR_PARAM(size_t, offset);
            INSTR_PARAM(size_t, size);
            ensure_size(&stack, &sp, size);
            struct Stack *stack2 = stack;
            u32 sp2 = sp;
            int i = 10;
            while (sp2 < offset + size && i > 0) {
                dbg(" sp2: %u\n offset: %lu\n size: %lu\n saved sp: %u\n\n", sp2, offset, size, stack2->saved_sp);
                offset -= sp2 + (STACK_CHUNK_SIZE - stack2->saved_sp);
                sp2 = stack2->saved_sp;
                stack2 = stack2->last;
                i--;
            }
            if (i == 0) {
                return 1;
            }
            memcpy(stack->data + sp, stack2->data + sp2 - offset - size, size);
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
            ensure_size(&stack, &sp, sizeof(handle));
            PUSH(Pointer, alloc_object(handle, size));
            break;
        }
        case 4: {
            dbg("alloca!\n");
            pc++;
            INSTR_PARAM(size_t, size);
            ensure_size(&stack, &sp, size);
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
            ensure_size(&stack, &sp, size);
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
            ensure_size(&stack, &sp, sizeof(lit));
            PUSH(i32, lit);
            break;
        }
        case 10: {
            dbg("global function!\n");
            pc++;
            INSTR_PARAM(u32, lit);
            ensure_size(&stack, &sp, sizeof(lit));
            PUSH(u32, lit);
            break;
        }
        case 11: {
            dbg("halt!\n");
            POP(u8, status_code);
            return status_code;
            break;
        }
        case 12: {
            dbg("new region!\n");
            pc++;
            INSTR_PARAM(size_t, size);
            Region *r = new_region(size);
            ensure_size(&stack, &sp, sizeof(r));
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
            ensure_size(&stack, &sp, size);
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
            ensure_size(&stack, &sp, sizeof(ptr));
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
                return 1;
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
                return 1;
            }
            ensure_size(&stack, &sp, elem_size);
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
            PUSH(i32, b / a);
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
                return 1;
            }
            ensure_size(&stack, &sp, elem_size);
            memcpy(stack->data + sp, ptr.reference + n, elem_size);
            sp += elem_size;
            break;
        }
        case 24: {
            dbg("copy n elements!\n");
            pc++;
            POP(i32, n);
            POP(Pointer, src_array);
            POP(Pointer, dest_array);
            INSTR_PARAM(size_t, elem_size);
            size_t size;
            u8 *src_ref;
            if (src_array.generation == -1) {
                // -1 generation means data section string
                size_t rest_of_data_section = instrs + 4 + data_section_size - src_array.reference;
                size = (size_t)n * elem_size;
                if (size > rest_of_data_section) {
                    size = rest_of_data_section;
                }
                src_ref = src_array.reference;
            } else {
                check_ptr(src_array);
                size_t array_len;
                memcpy(&array_len, src_array.reference, sizeof(array_len));
                size = (size_t)n * elem_size;
                if ((size_t)n > array_len) {
                    size = array_len * elem_size;
                }
                src_ref = src_array.reference + sizeof(array_len);
            }
            size_t dest_array_len;
            memcpy(&dest_array_len, dest_array.reference, sizeof(dest_array_len));
            if (n < 0) {
                printf("Runtime Error! Negative size (%d) during a copy.\n", n);
                return 1;
            } else if (dest_array_len < (u32)n) {
                printf("Runtime Error! Copy (%d) out of bounds for array of size %lu.\n", n, dest_array_len);
                return 1;
            }
            memcpy(dest_array.reference + sizeof(size), src_ref, size);
            PUSH(Pointer, dest_array);
            dbg("%.*s\n", (int)size, dest_array.reference + sizeof(size));
            break;
        }
        case 25: {
            dbg("u8 literal!\n");
            pc++;
            INSTR_PARAM(u8, val);
            ensure_size(&stack, &sp, sizeof(val));
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
            PUSH(u8, b / a);
            break;
        }
        case 29: {
            dbg("u8 to i32!\n");
            pc++;
            POP(u8, a);
            PUSH(i32, a);
            break;
        }
        case 30: {
            dbg("modulo i32!\n");
            pc++;
            POP(i32, a);
            POP(i32, b);
            PUSH(i32, b % a);
            break;
        }
        case 31: {
            dbg("modulo u8!\n");
            pc++;
            POP(u8, a);
            POP(u8, b);
            PUSH(u8, b % a);
            break;
        }
        case 32: {
            dbg("i32 to u8!\n");
            pc++;
            POP(i32, a);
            PUSH(u8, a);
            break;
        }
        case 33: {
            dbg("read!\n");
            pc++;
            INSTR_PARAM(u8, c);
            switch (c) {
                case 0: {
                    POP(Region*, r);
                    POP(Pointer, env);
                    POP(u32, handler);
                    stdin_handler.f = handler;
                    stdin_handler.env = env;
                    stdin_rgn = r;
                    waiting |= 0b1;
                    break;
                }
            }
            break;
        }
        case 34: {
            dbg("write!\n");
            pc++;
            INSTR_PARAM(u8, c);
            switch (c) {
                case 0: {
                    POP(Region*, r);
                    POP(u8, write_mode);
                    POP(Pointer, env);
                    POP(u32, handler);
                    POP(Pointer, str_ptr);
                    if (write_mode == 0) {
                        stdout_handler.f = handler;
                        stdout_handler.env = env;
                        size_t len;
                        memcpy(&len, str_ptr.reference, sizeof(len));
                        printf("%.*s", (int)len, str_ptr.reference + sizeof(len));
                        post_task(stdout_handler);
                    } else if (write_mode == 1) {
                        stderr_handler.f = handler;
                        stderr_handler.env = env;
                        size_t len;
                        memcpy(&len, str_ptr.reference, sizeof(len));
                        fprintf(stderr, "%.*s", (int)len, str_ptr.reference + sizeof(len));
                        post_task(stderr_handler);
                    } else {
                        printf("Internal SaberVM Error! Unknown write mode %d.\n", write_mode);
                        exit(1);
                    }
                    // waiting |= 0b10;
                    break;
                }
            }
            break;
        }
        default: {
            printf("internal error!! Unknown IR op %d, please let the SaberVM team know!!", instrs[pc]);
            return 1;
        }
        }
    }
}