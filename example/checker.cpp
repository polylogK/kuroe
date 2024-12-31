#include "testlib.h"

int main(int argc, char *argv[]) {
  registerTestlibCmd(argc, argv);

  auto output = ouf.readLine();
  auto answer = ans.readLine();

  if (output == answer)
    quit(_ok, "ok");
  else
    quit(_wa, "wa");
}
