c++ foo.cpp -c -o foo.o
ar rc libfoo.a foo.o
c++ bar.cpp libfoo.a -o main && ./main
#c++ bar.cpp -L. -lfoo -o main && ./main
