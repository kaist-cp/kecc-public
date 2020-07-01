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
}

extern "C" {
  int exotic_arguments_struct_small(model::small, int);
  long exotic_arguments_struct_large(model::large, int);
  float exotic_arguments_float(float, int);
  double exotic_arguments_double(double, int);
  int fibonacci_recursive(int, int);
  int fibonacci_loop(int, int);
  int two_dimension_array(int, int);
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

  std::vector<unsigned long> cycles;

  cycles.push_back(evaluate("exotic_arguments_struct_small", model::small { .a = 3, .b = 4 }, exotic_arguments_struct_small, model::exotic_arguments_struct_small));
  cycles.push_back(evaluate("exotic_arguments_struct_large", model::large { .a = 5, .b = 6, .c = 7, .d = 8, .e = 9, .f = 10, .g = 11, .h = 12 }, exotic_arguments_struct_large, model::exotic_arguments_struct_large));
  cycles.push_back(evaluate("exotic_arguments_float", 0.42f, exotic_arguments_float, model::exotic_arguments_float));
  cycles.push_back(evaluate("exotic_arguments_double", 0.42, exotic_arguments_double, model::exotic_arguments_double));
  cycles.push_back(evaluate("fibonacci_recursive", 30, fibonacci_recursive, model::fibonacci_recursive));
  cycles.push_back(evaluate("fibonacci_loop", 30, fibonacci_loop, model::fibonacci_loop));
  cycles.push_back(evaluate("two_dimension_array", 100, two_dimension_array, model::two_dimension_array));

  double average = 1.0;
  for (auto cycle: cycles) {
    average *= static_cast<double>(cycle);
  }
  average = std::pow(average, 1 / static_cast<double>(cycles.size()));

  std::cout << "[AVERAGE] " << average << std::endl;
  return 0;
}
