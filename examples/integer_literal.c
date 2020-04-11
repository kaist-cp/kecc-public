int main() {
    short temp = 0;
    unsigned int temp2 = 4294967163;
    return (char)(temp ^ temp2) == 123;
}
