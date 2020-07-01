CC=riscv64-linux-gnu-gcc-10
CXX=riscv64-linux-gnu-g++-10
KECC=../target/release/kecc
CFLAGS=-O
RM=rm -f

SRCS=$(shell find . -name "*.c")
OBJS=$(subst .c,.s,$(SRCS))

all: bench

bench: $(OBJS) driver.o
	$(CXX) -o bench $(OBJS) driver.o

run: bench
	qemu-riscv64-static -L /usr/riscv64-linux-gnu ./bench

driver.o: driver.cpp
	$(CXX) $(CFLAGS) -o driver.o -c -I. driver.cpp

.c.s:
	($(KECC) -O $< >$@) || (rm $@ -rf; exit 1)

$(KECC):
	cargo build --manifest-path=../Cargo.toml --release --bin kecc

clean:
	$(RM) $(OBJS) driver.o bench
