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

uint8_t vm_function(uint8_t bytes[]) {
    for (int i = 0; i < 20; i++) {
        dbg_str(" ");
        dbg_int(bytes[i]);
    }
    dbg_str("\n");
    uint32_t pc = 0;
    uint8_t sp = 0;
    uint32_t stack[1024];
    int call_count = 10;
    while (1) {
        dbg_str("pc: ");
        dbg_int(pc);
        dbg_str("\n");
        switch (bytes[pc]) {
            case 0: // get 
                stack[sp] = stack[sp-bytes[pc + 1]];
                sp += 1;
                pc += 2;
                break;
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
                pc = (uint32_t)(stack[--sp]);
                if (call_count-- == 0) {exit(0);}
                break;
            case 5: // print
                dbg_str("print!\n");
                printf("%d\n", stack[sp-1]);
                sp -= 1;
                pc += 1;
                break;
            case 6: // literal 
                dbg_str("literal!\n");
                {
                    unsigned int lit = bytes[pc + 1] << 24 | bytes[pc + 2] << 16 | bytes[pc + 3] << 8 | bytes[pc + 4];
                    stack[sp] = lit;
                    sp += 1;
                    pc += 5;
                }
                break;
            case 7: // global function
                dbg_str("global function!\n");
                {
                    unsigned int lit = bytes[pc + 1] << 24 | bytes[pc + 2] << 16 | bytes[pc + 3] << 8 | bytes[pc + 4];
                    stack[sp] = lit;
                    sp += 1;
                    pc += 5;
                }
                break;
            case 8: // halt
                dbg_str("halt!\n");
                exit(bytes[pc + 1]);
                break;
        }
    }
}