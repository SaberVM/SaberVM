#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>

#define DEBUG 0

void dbg_str(char *s) {
    #if DEBUG
    printf("%s", s);
    #endif
}

void dbg_int(int i) {
    #if DEBUG
    printf("%d", i);
    #endif
}

uint8_t vm_function(uint32_t instrs[], size_t instrs_len) {
    for (int i = 0; i < instrs_len; i++) {
        dbg_str(" ");
        dbg_int(instrs[i]);
    }
    dbg_str("\n");
    uint32_t pc = 0;
    uint8_t sp = 0;
    uint32_t stack[1024];
    while (1) {
        dbg_str("pc: ");
        dbg_int(pc);
        dbg_str("\n");
        switch (instrs[pc]) {
            case 0: // get 
                {
                    dbg_str("get!\n");
                    uint32_t offset = instrs[pc + 1];
                    uint32_t size = instrs[pc + 2];
                    stack[sp] = stack[sp-offset-size];
                    sp += size;
                    pc += 3;
                    break;
                }
            case 1: // init
                printf("init unimplemented\n");
                pc += 2;
                break;
            case 2: // malloc
                printf("malloc unimplemented\n");
                pc += 2;
                break;
            case 3: // projection
                printf("projection unimplemented\n");
                pc += 2;
                break;
            case 4: // call
                dbg_str("call!\n");
                pc = (uint32_t)(stack[sp-1]);
                sp -= 1;
                break;
            case 5: // print
                dbg_str("print!\n");
                printf("%d\n", stack[sp-1]);
                sp -= 1;
                pc += 1;
                break;
            case 6: // literal 
                {
                    dbg_str("literal!\n");
                    uint32_t lit = instrs[pc + 1];
                    stack[sp] = lit;
                    sp += 1;
                    pc += 2;
                }
                break;
            case 7: // global function
                {
                    dbg_str("global function!\n");
                    unsigned int lit = instrs[pc + 1];
                    stack[sp] = lit;
                    sp += 1;
                    pc += 2;
                }
                break;
            case 8: // halt
                dbg_str("halt!\n");
                exit(instrs[pc + 1]);
                break;
        }
    }
}