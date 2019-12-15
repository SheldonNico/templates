#include <iostream>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>

extern "C"
int double_input(int input) {
  std::cout << "Function is evaluated in cpp." << std::endl;
  auto console = spdlog::stdout_color_mt("console");
  console->error("hello from cpp 3rd library.");
  return input * 4;
}
