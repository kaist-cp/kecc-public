int fibonacci_loop(int n, int nonce) {
  int x = nonce;
  int y = nonce;

  for (int i = 1; i < n; ++i) {
    int newy = x + y;
    x = y;
    y = newy;
  }

  return y;
}

int fibonacci_recursive(int n, int nonce) {
  if (n < 2) {
    return nonce;
  }

  return fibonacci_recursive(n - 1, nonce) + fibonacci_recursive(n - 2, nonce);
}
