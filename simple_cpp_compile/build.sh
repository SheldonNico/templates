echo "static link"
c++ foo.cpp -c -o foo.o # use c to compile to object
ar rc libfoo.a foo.o
c++ bar.cpp libfoo.a -o main && ./main
#c++ bar.cpp -L. -lfoo -o main && ./main

echo "============================="

echo "Dynamic link"
c++ -fPIC -c foo.cpp -o foo.o # -fPIC is used to compile to shared library
c++ -shared -Wl,-soname,libfoo.so -o libfoo.so foo.o # use C++ pack *.o same as ar rc
#c++ bar.cpp libfoo.so -o main && ./main
c++ bar.cpp -L. -lfoo -o main && ./main
ldd main # use ldd to look link message
