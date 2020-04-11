int main() {
    int temp = 0;
    // `0xFFFFFFFF` is translated as `unsigned int` not `int`
    return temp < 0xFFFFFFFF;
}
