#include "string.h"

void* memset(void *s, int c, unsigned int len) {
    unsigned char *dst = s;
    while (len > 0) {
        *dst = (unsigned char) c;
        dst++;
        len--;
    }
    return s;
}


void memcpy(void *dest, void *src, unsigned int n)
{
// Typecast src and dest addresses to (char *)
char *csrc = (char *)src;
char *cdest = (char *)dest;

// Copy contents of src[] to dest[]
for (int i=0; i<n; i++) {    cdest[i] = csrc[i]; }

}

