void init(int row, int col, int a[4][5]) {
    for (int i = 0; i < row; i++) {
        for (int j = 0; j < col; j++) {
            a[i][j] = i * j;
        }
    }
}

int main() {
    int a[4][5];
    int row = 4, col = 5;

    init(row, col, a);

    return a[2][3] == 6;
}
