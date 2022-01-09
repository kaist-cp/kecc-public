CC=riscv64-linux-gnu-gcc
CXX=riscv64-linux-gnu-g++
KECC=../target/release/kecc
CFLAGS=-O
RM=rm -f

SRCS=$(shell find . -name "*.c")
OBJS=$(subst .c,.s,$(SRCS))
OBJS_GCC=$(subst .c,.o,$(SRCS))

all: bench bench-gcc

bench: $(OBJS) driver.o
	$(CXX) -o bench $(OBJS) driver.o

run: bench
	qemu-riscv64-static -L /usr/riscv64-linux-gnu ./bench

bench-gcc: $(OBJS_GCC) driver.o
	$(CXX) -o bench-gcc $(OBJS_GCC) driver.o

run-gcc: bench-gcc
	qemu-riscv64-static -L /usr/riscv64-linux-gnu ./bench-gcc

driver.o: $(SRCS) driver.cpp
	$(CXX) $(CFLAGS) -fsigned-char -o driver.o -c -I. driver.cpp

.c.s: $(KECC)
	($(KECC) -O $< >$@) || (rm $@ -rf; exit 1)

.c.o:
	($(CC) -O -fsigned-char -c $< -o $@) || (rm $@ -rf; exit 1)

$(KECC):
	cargo build --manifest-path=../Cargo.toml --release --bin kecc

clean:
	$(RM) $(OBJS) $(OBJS_GCC) driver.o bench bench-gcc
