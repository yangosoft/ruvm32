
void main()
{
     __asm__ volatile (
            "li a7, 64\n\t"   // llamada al sistema 64, write
            "ecall\n\t"
        );
    int x = 42;
}
