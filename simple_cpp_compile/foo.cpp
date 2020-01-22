#include "foo.h"

int foo(int a) {
  std::cout << "Invoke foo function by static library." << std::endl;
  return a+1;
}

