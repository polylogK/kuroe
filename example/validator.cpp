#include "testlib.h"

int main(int argc, char *argv[]) {
  registerValidation(argc, argv);

  auto s = inf.readLine();
  ensuref(s.substr(0, 7) == "example", "testcase must start with \"example\"");

  inf.readEof();
}
