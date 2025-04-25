#include <iostream>
#include <cstdlib>
#include <cmath>
#include <ctime>
#include <optional>
#include <vector>
#include <functional>

namespace model {
  #include <exotic_arguments.c>
  #include <fibonacci.c>
  #include <two_dimension_array.c>
  #include <matrix.c>
  #include <graph.c>
  #include <median.c>
  #include <multiply.c>
  #include <qsort.c>
  #include <rsort.c>
  #include <spmv.c>
  #include <towers.c>
  #include <vvadd.c>
}

extern "C" {
  int exotic_arguments_struct_small(model::small, int);
  long exotic_arguments_struct_large(model::large, int);
  float exotic_arguments_struct_small_ugly(model::small_ugly, int);
  double exotic_arguments_struct_large_ugly(model::large_ugly, int);
  float exotic_arguments_float(float, int);
  double exotic_arguments_double(double, int);
  int fibonacci_recursive(int, int);
  int fibonacci_loop(int, int);
  int two_dimension_array(int, int);
  int matrix_mul(int, int);
  int matrix_add(int, int);
  int graph_dijkstra(int, int);
  int graph_floyd_warshall(int, int);
  // From riscv-tests
  int run_median(int, int);
  int run_multiply(int, int);
  int run_qsort(int, int);
  int run_rsort(int, int);
  int run_spmv(int, int);
  int run_towers(int, int);
  int run_vvadd(int, int);
}

namespace {
  inline unsigned long read_cycles()
  {
    unsigned long cycles;
    asm volatile ("rdcycle %0" : "=r" (cycles));
    return cycles;
  }

  template<typename I, typename O>
  inline unsigned long evaluate(const char *name, I input, O (*solution)(I, int), O (*model)(I, int)) {
    std::cout << "[" << name << "] ";

    int nonce = 1 + (std::rand() % 100);
    auto start = read_cycles();
    auto output = solution(input, nonce);
    auto end = read_cycles();

    auto expected = model(input, nonce);
    if (output != expected) {
      std::cout << "mismatched result (expected: " << expected << ", actual: " << output << ")" << std::endl;
      std::exit(1);
    }

    auto cycles = end - start;
    std::cout << cycles << std::endl;
    return cycles;
  }
}

int main() {
  std::srand(static_cast<unsigned>(time(NULL)));

  // Checks if the compiler observes the calling convention.
  evaluate("exotic_arguments_struct_small", model::small { .a = 3, .b = 4 }, exotic_arguments_struct_small, model::exotic_arguments_struct_small);
  evaluate("exotic_arguments_struct_large", model::large { .a = 5, .b = 6, .c = 7, .d = 8, .e = 9, .f = 10, .g = 11, .h = 12 }, exotic_arguments_struct_large, model::exotic_arguments_struct_large);
  evaluate("exotic_arguments_struct_small_ugly", model::small_ugly { .a = 5, .b = 6.0f }, exotic_arguments_struct_small_ugly, model::exotic_arguments_struct_small_ugly);
  evaluate("exotic_arguments_struct_large_ugly", model::large_ugly { .a = 5, .b = 6.0f, .c = 7, .d = 8.0, .e = 9, .f = 10, .g = 11, .h = 12.0, .i = 13, .j = 14, .k = 15, .l = 16.0 }, exotic_arguments_struct_large_ugly, model::exotic_arguments_struct_large_ugly);
  evaluate("exotic_arguments_float", 0.42f, exotic_arguments_float, model::exotic_arguments_float);
  evaluate("exotic_arguments_double", 0.42, exotic_arguments_double, model::exotic_arguments_double);

  // Measures cycles for computationally heavy programs.
  std::vector<unsigned long> cycles;
  for (int i = 0; i < 10; ++i) {
    cycles.push_back(evaluate("fibonacci_recursive", 30, fibonacci_recursive, model::fibonacci_recursive));
    cycles.push_back(evaluate("fibonacci_loop", 30, fibonacci_loop, model::fibonacci_loop));
    cycles.push_back(evaluate("two_dimension_array", 100, two_dimension_array, model::two_dimension_array));
    cycles.push_back(evaluate("matrix_mul", 30, matrix_mul, model::matrix_mul));
    cycles.push_back(evaluate("matrix_add", 30, matrix_add, model::matrix_add));
    cycles.push_back(evaluate("graph_dijkstra", 1000, graph_dijkstra, model::graph_dijkstra));
    cycles.push_back(evaluate("graph_floyd_warshall", 200, graph_floyd_warshall, model::graph_floyd_warshall));
    cycles.push_back(evaluate("median", -1, run_median, model::run_median));
    cycles.push_back(evaluate("mutiply", -1, run_multiply, model::run_multiply));
    cycles.push_back(evaluate("qsort", -1, run_qsort, model::run_qsort));
    cycles.push_back(evaluate("rsort", -1, run_rsort, model::run_rsort));
    cycles.push_back(evaluate("spmv", -1, run_spmv, model::run_spmv));
    cycles.push_back(evaluate("towers", -1, run_towers, model::run_towers));
    cycles.push_back(evaluate("vvadd", -1, run_vvadd, model::run_vvadd));
  }

  // Calculates the geometric mean.
  auto average = 1.0;
  auto factor = 1 / static_cast<double>(cycles.size());
  for (auto cycle: cycles) {
    average *= std::pow(static_cast<double>(cycle), factor);
  }

  std::cout << "[AVERAGE] " << average << std::endl;
  return 0;
}
