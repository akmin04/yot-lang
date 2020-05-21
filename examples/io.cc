#include <iostream>

extern "C" {
int println(int a) {
  std::cout << a << std::endl;
  return 0;
}

int next_int() {
  int input;
  std::cin >> input;
  return input;
}
}