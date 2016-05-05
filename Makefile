ifndef PREFIX
	PREFIX=${HOME}
endif

CC=gcc
CFLAGS=-g3 -Wall -Wextra -std=c99 -O2 -fPIC -Iinclude
OFILES=src/lightfield_optics.o \
	   src/lightfield_angular_plane.o \
	   src/lightfield_plane_geometry.o 

.PHONY: tex clean

all: liblightfield.so tex install_lib

install_lib: liblightfield.so
	mkdir -p ${PREFIX}/lib
	mkdir -p ${PREFIX}/include/
	cp liblightfield.so ${PREFIX}/lib
	cp -r include/lightfield ${PREFIX}/include

liblightfield.so: ${OFILES}
	${CC} ${CFLAGS} -lm -shared -o $@ $^

tex:
	$(MAKE) -C tex

clean:
	$(MAKE) -C tex clean
	rm ${OFILES} liblightfield.so

