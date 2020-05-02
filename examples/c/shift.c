int main() {
    char a = 127;
    char b = a << 1;
    unsigned char c = (unsigned char)b >> 1;

    return b == -2 && c == 0x7F;
}
