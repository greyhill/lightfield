ifndef PREFIX
	PREFIX=${HOME}
endif

ifndef CC
	CC=gcc
endif
CFLAGS=-g3 -Wall -Wextra -std=c99 -O2 -fPIC -Iinclude
OFILES=src/lightfield_optics.o \
	   src/lightfield_angular_plane.o \
	   src/lightfield_plane_geometry.o \
	   src/lightfield_lixel.o \
	   src/lightfield_transport.o

.PHONY: tex clean install_lib install_python

all: liblightfield.so tex 

install: install_lib install_python

install_lib: liblightfield.so
	mkdir -p ${PREFIX}/lib
	mkdir -p ${PREFIX}/include/
	cp liblightfield.so ${PREFIX}/lib
	cp -r include/lightfield ${PREFIX}/include

liblightfield.so: ${OFILES}
	${CC} ${CFLAGS} -lm -shared -o $@ $^

tex:
	$(MAKE) -C tex

install_python:
	pip install --upgrade py/

clean:
	$(MAKE) -C tex clean
	rm ${OFILES} liblightfield.so

