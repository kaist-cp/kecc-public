typedef int i32_t;
typedef i32_t* p_i32_t;

int main() {
    i32_t a = 0;
    p_i32_t const b = &a;
    *b = 1;

    return *b;
}
