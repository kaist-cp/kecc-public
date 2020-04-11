int main() {
    unsigned char a = -1;
    unsigned char b = -128;
    unsigned char c = 127;
    unsigned char d = b | a; // -1 (255)
    unsigned char e = b & a; // -128 (128)
    unsigned char f = b & c; // 0 (0)
    unsigned char g = b | c; // -1 (255)
    unsigned char h = -1 ^ -1; // 0 (0)
    unsigned char i = -1 ^ 0; // -1 (255)
    
    return d == 255 && e == 128 && f == 0 && g == 255 && h == 0 && i == 255;
}
