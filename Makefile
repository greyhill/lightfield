UNAME = $(shell uname)

ifndef PREFIX
	PREFIX=${HOME}
endif

ifndef CC
	CC=gcc
endif

ifeq ($(UNAME), Darwin)
	LIBFLAGS=-framework OpenCL
endif

ifeq ($(UNAME), Linux)
	LIBFLAGS=-lOpenCL
endif

CFLAGS=-g3 -Wall -Wextra -std=c99 -O2 -fPIC -Iinclude
OFILES=src/lightfield.o

.PHONY: tex clean install_lib install_python
.SUFFIXES: .opencl .clo

.opencl.clo:
	xxd -i $^ | gcc -c -xc -fPIC -o $@ -

all: lib 

lib: liblightfield.so

install: install_lib install_python

install_lib: liblightfield.so
	mkdir -p ${PREFIX}/lib
	mkdir -p ${PREFIX}/include/
	cp liblightfield.so ${PREFIX}/lib
	cp -r include/lightfield ${PREFIX}/include

liblightfield.so: ${OFILES}
	${CC} ${CFLAGS} ${LIBFLAGS} -lm -shared -o $@ $^

tex:
	$(MAKE) -C tex

install_python:
	pip install --upgrade py/

clean:
	$(MAKE) -C tex clean
	rm ${OFILES} liblightfield.so

