typedef struct {
  int a;
  int b;
} small;

typedef struct {
  long a;
  long b;
  long c;
  long d;
  long e;
  long f;
  long g;
  long h;
} large;

int exotic_arguments_struct_small(small a, int nonce) {
  return a.a + a.b + nonce;
}

long exotic_arguments_struct_large(large a, int nonce) {
  return a.a + a.b + a.c + a.d + a.e + a.f + a.g + a.h + nonce;
}

float exotic_arguments_float(float a, int nonce) {
  return a + (float) nonce;
}

double exotic_arguments_double(double a, int nonce) {
  return a + (double) nonce;
}
