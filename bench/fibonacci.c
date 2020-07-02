int fibonacci_loop(int n, int nonce) {
  int result = 0;

  for (int step = 0; step < 10; ++step) {
    int x = nonce;
    int y = nonce;

    for (int i = 1; i < n; ++i) {
      int newy = x + y;
      newy += (x + y);
      newy += (x + y);
      newy += (x + y);
      newy += (x + y);
      newy += (x + y);
      newy -= (x + y);
      newy -= (x + y);
      newy -= (x + y);
      newy -= (x + y);
      newy -= (x + y);
      x = y;
      y = newy;
    }

    result += y;
  }

  return result;
}

int fibonacci_recursive(int n, int nonce) {
  if (n < 2) {
    return nonce;
  }

  return fibonacci_recursive(n - 1, nonce) + fibonacci_recursive(n - 2, nonce);
}
