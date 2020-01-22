#include "foo.h"

int main() {
  int i = foo(2);
  std::cout << "Call in main" << std::endl;
  return i;
}
