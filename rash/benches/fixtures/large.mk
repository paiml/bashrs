# Large Makefile (~1000 lines, 100 targets)
# Sprint 84 Performance Benchmark - Large Fixture

.PHONY: all clean test install

CC = gcc
CFLAGS = -Wall -Wextra -O2
PREFIX = /usr/local
BUILD_DIR = build
SRC_DIR = src

# All modules

# Module 1
MODULE1_SRC = $(SRC_DIR)/module1.c
MODULE1_OBJ = $(BUILD_DIR)/module1.o
MODULE1_LIB = $(BUILD_DIR)/libmodule1.a

$(MODULE1_OBJ): $(MODULE1_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE1_LIB): $(MODULE1_OBJ)
	ar rcs $@ $<

module1: $(MODULE1_LIB)

module1-test: $(MODULE1_LIB)
	@echo "Testing module 1"
	@./test_module1

module1-clean:
	rm -f $(MODULE1_OBJ) $(MODULE1_LIB)

# Module 2
MODULE2_SRC = $(SRC_DIR)/module2.c
MODULE2_OBJ = $(BUILD_DIR)/module2.o
MODULE2_LIB = $(BUILD_DIR)/libmodule2.a

$(MODULE2_OBJ): $(MODULE2_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE2_LIB): $(MODULE2_OBJ)
	ar rcs $@ $<

module2: $(MODULE2_LIB)

module2-test: $(MODULE2_LIB)
	@echo "Testing module 2"
	@./test_module2

module2-clean:
	rm -f $(MODULE2_OBJ) $(MODULE2_LIB)

# Module 3
MODULE3_SRC = $(SRC_DIR)/module3.c
MODULE3_OBJ = $(BUILD_DIR)/module3.o
MODULE3_LIB = $(BUILD_DIR)/libmodule3.a

$(MODULE3_OBJ): $(MODULE3_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE3_LIB): $(MODULE3_OBJ)
	ar rcs $@ $<

module3: $(MODULE3_LIB)

module3-test: $(MODULE3_LIB)
	@echo "Testing module 3"
	@./test_module3

module3-clean:
	rm -f $(MODULE3_OBJ) $(MODULE3_LIB)

# Module 4
MODULE4_SRC = $(SRC_DIR)/module4.c
MODULE4_OBJ = $(BUILD_DIR)/module4.o
MODULE4_LIB = $(BUILD_DIR)/libmodule4.a

$(MODULE4_OBJ): $(MODULE4_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE4_LIB): $(MODULE4_OBJ)
	ar rcs $@ $<

module4: $(MODULE4_LIB)

module4-test: $(MODULE4_LIB)
	@echo "Testing module 4"
	@./test_module4

module4-clean:
	rm -f $(MODULE4_OBJ) $(MODULE4_LIB)

# Module 5
MODULE5_SRC = $(SRC_DIR)/module5.c
MODULE5_OBJ = $(BUILD_DIR)/module5.o
MODULE5_LIB = $(BUILD_DIR)/libmodule5.a

$(MODULE5_OBJ): $(MODULE5_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE5_LIB): $(MODULE5_OBJ)
	ar rcs $@ $<

module5: $(MODULE5_LIB)

module5-test: $(MODULE5_LIB)
	@echo "Testing module 5"
	@./test_module5

module5-clean:
	rm -f $(MODULE5_OBJ) $(MODULE5_LIB)

# Module 6
MODULE6_SRC = $(SRC_DIR)/module6.c
MODULE6_OBJ = $(BUILD_DIR)/module6.o
MODULE6_LIB = $(BUILD_DIR)/libmodule6.a

$(MODULE6_OBJ): $(MODULE6_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE6_LIB): $(MODULE6_OBJ)
	ar rcs $@ $<

module6: $(MODULE6_LIB)

module6-test: $(MODULE6_LIB)
	@echo "Testing module 6"
	@./test_module6

module6-clean:
	rm -f $(MODULE6_OBJ) $(MODULE6_LIB)

# Module 7
MODULE7_SRC = $(SRC_DIR)/module7.c
MODULE7_OBJ = $(BUILD_DIR)/module7.o
MODULE7_LIB = $(BUILD_DIR)/libmodule7.a

$(MODULE7_OBJ): $(MODULE7_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE7_LIB): $(MODULE7_OBJ)
	ar rcs $@ $<

module7: $(MODULE7_LIB)

module7-test: $(MODULE7_LIB)
	@echo "Testing module 7"
	@./test_module7

module7-clean:
	rm -f $(MODULE7_OBJ) $(MODULE7_LIB)

# Module 8
MODULE8_SRC = $(SRC_DIR)/module8.c
MODULE8_OBJ = $(BUILD_DIR)/module8.o
MODULE8_LIB = $(BUILD_DIR)/libmodule8.a

$(MODULE8_OBJ): $(MODULE8_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE8_LIB): $(MODULE8_OBJ)
	ar rcs $@ $<

module8: $(MODULE8_LIB)

module8-test: $(MODULE8_LIB)
	@echo "Testing module 8"
	@./test_module8

module8-clean:
	rm -f $(MODULE8_OBJ) $(MODULE8_LIB)

# Module 9
MODULE9_SRC = $(SRC_DIR)/module9.c
MODULE9_OBJ = $(BUILD_DIR)/module9.o
MODULE9_LIB = $(BUILD_DIR)/libmodule9.a

$(MODULE9_OBJ): $(MODULE9_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE9_LIB): $(MODULE9_OBJ)
	ar rcs $@ $<

module9: $(MODULE9_LIB)

module9-test: $(MODULE9_LIB)
	@echo "Testing module 9"
	@./test_module9

module9-clean:
	rm -f $(MODULE9_OBJ) $(MODULE9_LIB)

# Module 10
MODULE10_SRC = $(SRC_DIR)/module10.c
MODULE10_OBJ = $(BUILD_DIR)/module10.o
MODULE10_LIB = $(BUILD_DIR)/libmodule10.a

$(MODULE10_OBJ): $(MODULE10_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE10_LIB): $(MODULE10_OBJ)
	ar rcs $@ $<

module10: $(MODULE10_LIB)

module10-test: $(MODULE10_LIB)
	@echo "Testing module 10"
	@./test_module10

module10-clean:
	rm -f $(MODULE10_OBJ) $(MODULE10_LIB)

# Module 11
MODULE11_SRC = $(SRC_DIR)/module11.c
MODULE11_OBJ = $(BUILD_DIR)/module11.o
MODULE11_LIB = $(BUILD_DIR)/libmodule11.a

$(MODULE11_OBJ): $(MODULE11_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE11_LIB): $(MODULE11_OBJ)
	ar rcs $@ $<

module11: $(MODULE11_LIB)

module11-test: $(MODULE11_LIB)
	@echo "Testing module 11"
	@./test_module11

module11-clean:
	rm -f $(MODULE11_OBJ) $(MODULE11_LIB)

# Module 12
MODULE12_SRC = $(SRC_DIR)/module12.c
MODULE12_OBJ = $(BUILD_DIR)/module12.o
MODULE12_LIB = $(BUILD_DIR)/libmodule12.a

$(MODULE12_OBJ): $(MODULE12_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE12_LIB): $(MODULE12_OBJ)
	ar rcs $@ $<

module12: $(MODULE12_LIB)

module12-test: $(MODULE12_LIB)
	@echo "Testing module 12"
	@./test_module12

module12-clean:
	rm -f $(MODULE12_OBJ) $(MODULE12_LIB)

# Module 13
MODULE13_SRC = $(SRC_DIR)/module13.c
MODULE13_OBJ = $(BUILD_DIR)/module13.o
MODULE13_LIB = $(BUILD_DIR)/libmodule13.a

$(MODULE13_OBJ): $(MODULE13_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE13_LIB): $(MODULE13_OBJ)
	ar rcs $@ $<

module13: $(MODULE13_LIB)

module13-test: $(MODULE13_LIB)
	@echo "Testing module 13"
	@./test_module13

module13-clean:
	rm -f $(MODULE13_OBJ) $(MODULE13_LIB)

# Module 14
MODULE14_SRC = $(SRC_DIR)/module14.c
MODULE14_OBJ = $(BUILD_DIR)/module14.o
MODULE14_LIB = $(BUILD_DIR)/libmodule14.a

$(MODULE14_OBJ): $(MODULE14_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE14_LIB): $(MODULE14_OBJ)
	ar rcs $@ $<

module14: $(MODULE14_LIB)

module14-test: $(MODULE14_LIB)
	@echo "Testing module 14"
	@./test_module14

module14-clean:
	rm -f $(MODULE14_OBJ) $(MODULE14_LIB)

# Module 15
MODULE15_SRC = $(SRC_DIR)/module15.c
MODULE15_OBJ = $(BUILD_DIR)/module15.o
MODULE15_LIB = $(BUILD_DIR)/libmodule15.a

$(MODULE15_OBJ): $(MODULE15_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE15_LIB): $(MODULE15_OBJ)
	ar rcs $@ $<

module15: $(MODULE15_LIB)

module15-test: $(MODULE15_LIB)
	@echo "Testing module 15"
	@./test_module15

module15-clean:
	rm -f $(MODULE15_OBJ) $(MODULE15_LIB)

# Module 16
MODULE16_SRC = $(SRC_DIR)/module16.c
MODULE16_OBJ = $(BUILD_DIR)/module16.o
MODULE16_LIB = $(BUILD_DIR)/libmodule16.a

$(MODULE16_OBJ): $(MODULE16_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE16_LIB): $(MODULE16_OBJ)
	ar rcs $@ $<

module16: $(MODULE16_LIB)

module16-test: $(MODULE16_LIB)
	@echo "Testing module 16"
	@./test_module16

module16-clean:
	rm -f $(MODULE16_OBJ) $(MODULE16_LIB)

# Module 17
MODULE17_SRC = $(SRC_DIR)/module17.c
MODULE17_OBJ = $(BUILD_DIR)/module17.o
MODULE17_LIB = $(BUILD_DIR)/libmodule17.a

$(MODULE17_OBJ): $(MODULE17_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE17_LIB): $(MODULE17_OBJ)
	ar rcs $@ $<

module17: $(MODULE17_LIB)

module17-test: $(MODULE17_LIB)
	@echo "Testing module 17"
	@./test_module17

module17-clean:
	rm -f $(MODULE17_OBJ) $(MODULE17_LIB)

# Module 18
MODULE18_SRC = $(SRC_DIR)/module18.c
MODULE18_OBJ = $(BUILD_DIR)/module18.o
MODULE18_LIB = $(BUILD_DIR)/libmodule18.a

$(MODULE18_OBJ): $(MODULE18_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE18_LIB): $(MODULE18_OBJ)
	ar rcs $@ $<

module18: $(MODULE18_LIB)

module18-test: $(MODULE18_LIB)
	@echo "Testing module 18"
	@./test_module18

module18-clean:
	rm -f $(MODULE18_OBJ) $(MODULE18_LIB)

# Module 19
MODULE19_SRC = $(SRC_DIR)/module19.c
MODULE19_OBJ = $(BUILD_DIR)/module19.o
MODULE19_LIB = $(BUILD_DIR)/libmodule19.a

$(MODULE19_OBJ): $(MODULE19_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE19_LIB): $(MODULE19_OBJ)
	ar rcs $@ $<

module19: $(MODULE19_LIB)

module19-test: $(MODULE19_LIB)
	@echo "Testing module 19"
	@./test_module19

module19-clean:
	rm -f $(MODULE19_OBJ) $(MODULE19_LIB)

# Module 20
MODULE20_SRC = $(SRC_DIR)/module20.c
MODULE20_OBJ = $(BUILD_DIR)/module20.o
MODULE20_LIB = $(BUILD_DIR)/libmodule20.a

$(MODULE20_OBJ): $(MODULE20_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE20_LIB): $(MODULE20_OBJ)
	ar rcs $@ $<

module20: $(MODULE20_LIB)

module20-test: $(MODULE20_LIB)
	@echo "Testing module 20"
	@./test_module20

module20-clean:
	rm -f $(MODULE20_OBJ) $(MODULE20_LIB)

# Module 21
MODULE21_SRC = $(SRC_DIR)/module21.c
MODULE21_OBJ = $(BUILD_DIR)/module21.o
MODULE21_LIB = $(BUILD_DIR)/libmodule21.a

$(MODULE21_OBJ): $(MODULE21_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE21_LIB): $(MODULE21_OBJ)
	ar rcs $@ $<

module21: $(MODULE21_LIB)

module21-test: $(MODULE21_LIB)
	@echo "Testing module 21"
	@./test_module21

module21-clean:
	rm -f $(MODULE21_OBJ) $(MODULE21_LIB)

# Module 22
MODULE22_SRC = $(SRC_DIR)/module22.c
MODULE22_OBJ = $(BUILD_DIR)/module22.o
MODULE22_LIB = $(BUILD_DIR)/libmodule22.a

$(MODULE22_OBJ): $(MODULE22_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE22_LIB): $(MODULE22_OBJ)
	ar rcs $@ $<

module22: $(MODULE22_LIB)

module22-test: $(MODULE22_LIB)
	@echo "Testing module 22"
	@./test_module22

module22-clean:
	rm -f $(MODULE22_OBJ) $(MODULE22_LIB)

# Module 23
MODULE23_SRC = $(SRC_DIR)/module23.c
MODULE23_OBJ = $(BUILD_DIR)/module23.o
MODULE23_LIB = $(BUILD_DIR)/libmodule23.a

$(MODULE23_OBJ): $(MODULE23_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE23_LIB): $(MODULE23_OBJ)
	ar rcs $@ $<

module23: $(MODULE23_LIB)

module23-test: $(MODULE23_LIB)
	@echo "Testing module 23"
	@./test_module23

module23-clean:
	rm -f $(MODULE23_OBJ) $(MODULE23_LIB)

# Module 24
MODULE24_SRC = $(SRC_DIR)/module24.c
MODULE24_OBJ = $(BUILD_DIR)/module24.o
MODULE24_LIB = $(BUILD_DIR)/libmodule24.a

$(MODULE24_OBJ): $(MODULE24_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE24_LIB): $(MODULE24_OBJ)
	ar rcs $@ $<

module24: $(MODULE24_LIB)

module24-test: $(MODULE24_LIB)
	@echo "Testing module 24"
	@./test_module24

module24-clean:
	rm -f $(MODULE24_OBJ) $(MODULE24_LIB)

# Module 25
MODULE25_SRC = $(SRC_DIR)/module25.c
MODULE25_OBJ = $(BUILD_DIR)/module25.o
MODULE25_LIB = $(BUILD_DIR)/libmodule25.a

$(MODULE25_OBJ): $(MODULE25_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE25_LIB): $(MODULE25_OBJ)
	ar rcs $@ $<

module25: $(MODULE25_LIB)

module25-test: $(MODULE25_LIB)
	@echo "Testing module 25"
	@./test_module25

module25-clean:
	rm -f $(MODULE25_OBJ) $(MODULE25_LIB)

# Module 26
MODULE26_SRC = $(SRC_DIR)/module26.c
MODULE26_OBJ = $(BUILD_DIR)/module26.o
MODULE26_LIB = $(BUILD_DIR)/libmodule26.a

$(MODULE26_OBJ): $(MODULE26_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE26_LIB): $(MODULE26_OBJ)
	ar rcs $@ $<

module26: $(MODULE26_LIB)

module26-test: $(MODULE26_LIB)
	@echo "Testing module 26"
	@./test_module26

module26-clean:
	rm -f $(MODULE26_OBJ) $(MODULE26_LIB)

# Module 27
MODULE27_SRC = $(SRC_DIR)/module27.c
MODULE27_OBJ = $(BUILD_DIR)/module27.o
MODULE27_LIB = $(BUILD_DIR)/libmodule27.a

$(MODULE27_OBJ): $(MODULE27_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE27_LIB): $(MODULE27_OBJ)
	ar rcs $@ $<

module27: $(MODULE27_LIB)

module27-test: $(MODULE27_LIB)
	@echo "Testing module 27"
	@./test_module27

module27-clean:
	rm -f $(MODULE27_OBJ) $(MODULE27_LIB)

# Module 28
MODULE28_SRC = $(SRC_DIR)/module28.c
MODULE28_OBJ = $(BUILD_DIR)/module28.o
MODULE28_LIB = $(BUILD_DIR)/libmodule28.a

$(MODULE28_OBJ): $(MODULE28_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE28_LIB): $(MODULE28_OBJ)
	ar rcs $@ $<

module28: $(MODULE28_LIB)

module28-test: $(MODULE28_LIB)
	@echo "Testing module 28"
	@./test_module28

module28-clean:
	rm -f $(MODULE28_OBJ) $(MODULE28_LIB)

# Module 29
MODULE29_SRC = $(SRC_DIR)/module29.c
MODULE29_OBJ = $(BUILD_DIR)/module29.o
MODULE29_LIB = $(BUILD_DIR)/libmodule29.a

$(MODULE29_OBJ): $(MODULE29_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE29_LIB): $(MODULE29_OBJ)
	ar rcs $@ $<

module29: $(MODULE29_LIB)

module29-test: $(MODULE29_LIB)
	@echo "Testing module 29"
	@./test_module29

module29-clean:
	rm -f $(MODULE29_OBJ) $(MODULE29_LIB)

# Module 30
MODULE30_SRC = $(SRC_DIR)/module30.c
MODULE30_OBJ = $(BUILD_DIR)/module30.o
MODULE30_LIB = $(BUILD_DIR)/libmodule30.a

$(MODULE30_OBJ): $(MODULE30_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE30_LIB): $(MODULE30_OBJ)
	ar rcs $@ $<

module30: $(MODULE30_LIB)

module30-test: $(MODULE30_LIB)
	@echo "Testing module 30"
	@./test_module30

module30-clean:
	rm -f $(MODULE30_OBJ) $(MODULE30_LIB)

# Module 31
MODULE31_SRC = $(SRC_DIR)/module31.c
MODULE31_OBJ = $(BUILD_DIR)/module31.o
MODULE31_LIB = $(BUILD_DIR)/libmodule31.a

$(MODULE31_OBJ): $(MODULE31_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE31_LIB): $(MODULE31_OBJ)
	ar rcs $@ $<

module31: $(MODULE31_LIB)

module31-test: $(MODULE31_LIB)
	@echo "Testing module 31"
	@./test_module31

module31-clean:
	rm -f $(MODULE31_OBJ) $(MODULE31_LIB)

# Module 32
MODULE32_SRC = $(SRC_DIR)/module32.c
MODULE32_OBJ = $(BUILD_DIR)/module32.o
MODULE32_LIB = $(BUILD_DIR)/libmodule32.a

$(MODULE32_OBJ): $(MODULE32_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE32_LIB): $(MODULE32_OBJ)
	ar rcs $@ $<

module32: $(MODULE32_LIB)

module32-test: $(MODULE32_LIB)
	@echo "Testing module 32"
	@./test_module32

module32-clean:
	rm -f $(MODULE32_OBJ) $(MODULE32_LIB)

# Module 33
MODULE33_SRC = $(SRC_DIR)/module33.c
MODULE33_OBJ = $(BUILD_DIR)/module33.o
MODULE33_LIB = $(BUILD_DIR)/libmodule33.a

$(MODULE33_OBJ): $(MODULE33_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE33_LIB): $(MODULE33_OBJ)
	ar rcs $@ $<

module33: $(MODULE33_LIB)

module33-test: $(MODULE33_LIB)
	@echo "Testing module 33"
	@./test_module33

module33-clean:
	rm -f $(MODULE33_OBJ) $(MODULE33_LIB)

# Module 34
MODULE34_SRC = $(SRC_DIR)/module34.c
MODULE34_OBJ = $(BUILD_DIR)/module34.o
MODULE34_LIB = $(BUILD_DIR)/libmodule34.a

$(MODULE34_OBJ): $(MODULE34_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE34_LIB): $(MODULE34_OBJ)
	ar rcs $@ $<

module34: $(MODULE34_LIB)

module34-test: $(MODULE34_LIB)
	@echo "Testing module 34"
	@./test_module34

module34-clean:
	rm -f $(MODULE34_OBJ) $(MODULE34_LIB)

# Module 35
MODULE35_SRC = $(SRC_DIR)/module35.c
MODULE35_OBJ = $(BUILD_DIR)/module35.o
MODULE35_LIB = $(BUILD_DIR)/libmodule35.a

$(MODULE35_OBJ): $(MODULE35_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE35_LIB): $(MODULE35_OBJ)
	ar rcs $@ $<

module35: $(MODULE35_LIB)

module35-test: $(MODULE35_LIB)
	@echo "Testing module 35"
	@./test_module35

module35-clean:
	rm -f $(MODULE35_OBJ) $(MODULE35_LIB)

# Module 36
MODULE36_SRC = $(SRC_DIR)/module36.c
MODULE36_OBJ = $(BUILD_DIR)/module36.o
MODULE36_LIB = $(BUILD_DIR)/libmodule36.a

$(MODULE36_OBJ): $(MODULE36_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE36_LIB): $(MODULE36_OBJ)
	ar rcs $@ $<

module36: $(MODULE36_LIB)

module36-test: $(MODULE36_LIB)
	@echo "Testing module 36"
	@./test_module36

module36-clean:
	rm -f $(MODULE36_OBJ) $(MODULE36_LIB)

# Module 37
MODULE37_SRC = $(SRC_DIR)/module37.c
MODULE37_OBJ = $(BUILD_DIR)/module37.o
MODULE37_LIB = $(BUILD_DIR)/libmodule37.a

$(MODULE37_OBJ): $(MODULE37_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE37_LIB): $(MODULE37_OBJ)
	ar rcs $@ $<

module37: $(MODULE37_LIB)

module37-test: $(MODULE37_LIB)
	@echo "Testing module 37"
	@./test_module37

module37-clean:
	rm -f $(MODULE37_OBJ) $(MODULE37_LIB)

# Module 38
MODULE38_SRC = $(SRC_DIR)/module38.c
MODULE38_OBJ = $(BUILD_DIR)/module38.o
MODULE38_LIB = $(BUILD_DIR)/libmodule38.a

$(MODULE38_OBJ): $(MODULE38_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE38_LIB): $(MODULE38_OBJ)
	ar rcs $@ $<

module38: $(MODULE38_LIB)

module38-test: $(MODULE38_LIB)
	@echo "Testing module 38"
	@./test_module38

module38-clean:
	rm -f $(MODULE38_OBJ) $(MODULE38_LIB)

# Module 39
MODULE39_SRC = $(SRC_DIR)/module39.c
MODULE39_OBJ = $(BUILD_DIR)/module39.o
MODULE39_LIB = $(BUILD_DIR)/libmodule39.a

$(MODULE39_OBJ): $(MODULE39_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE39_LIB): $(MODULE39_OBJ)
	ar rcs $@ $<

module39: $(MODULE39_LIB)

module39-test: $(MODULE39_LIB)
	@echo "Testing module 39"
	@./test_module39

module39-clean:
	rm -f $(MODULE39_OBJ) $(MODULE39_LIB)

# Module 40
MODULE40_SRC = $(SRC_DIR)/module40.c
MODULE40_OBJ = $(BUILD_DIR)/module40.o
MODULE40_LIB = $(BUILD_DIR)/libmodule40.a

$(MODULE40_OBJ): $(MODULE40_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE40_LIB): $(MODULE40_OBJ)
	ar rcs $@ $<

module40: $(MODULE40_LIB)

module40-test: $(MODULE40_LIB)
	@echo "Testing module 40"
	@./test_module40

module40-clean:
	rm -f $(MODULE40_OBJ) $(MODULE40_LIB)

# Module 41
MODULE41_SRC = $(SRC_DIR)/module41.c
MODULE41_OBJ = $(BUILD_DIR)/module41.o
MODULE41_LIB = $(BUILD_DIR)/libmodule41.a

$(MODULE41_OBJ): $(MODULE41_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE41_LIB): $(MODULE41_OBJ)
	ar rcs $@ $<

module41: $(MODULE41_LIB)

module41-test: $(MODULE41_LIB)
	@echo "Testing module 41"
	@./test_module41

module41-clean:
	rm -f $(MODULE41_OBJ) $(MODULE41_LIB)

# Module 42
MODULE42_SRC = $(SRC_DIR)/module42.c
MODULE42_OBJ = $(BUILD_DIR)/module42.o
MODULE42_LIB = $(BUILD_DIR)/libmodule42.a

$(MODULE42_OBJ): $(MODULE42_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE42_LIB): $(MODULE42_OBJ)
	ar rcs $@ $<

module42: $(MODULE42_LIB)

module42-test: $(MODULE42_LIB)
	@echo "Testing module 42"
	@./test_module42

module42-clean:
	rm -f $(MODULE42_OBJ) $(MODULE42_LIB)

# Module 43
MODULE43_SRC = $(SRC_DIR)/module43.c
MODULE43_OBJ = $(BUILD_DIR)/module43.o
MODULE43_LIB = $(BUILD_DIR)/libmodule43.a

$(MODULE43_OBJ): $(MODULE43_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE43_LIB): $(MODULE43_OBJ)
	ar rcs $@ $<

module43: $(MODULE43_LIB)

module43-test: $(MODULE43_LIB)
	@echo "Testing module 43"
	@./test_module43

module43-clean:
	rm -f $(MODULE43_OBJ) $(MODULE43_LIB)

# Module 44
MODULE44_SRC = $(SRC_DIR)/module44.c
MODULE44_OBJ = $(BUILD_DIR)/module44.o
MODULE44_LIB = $(BUILD_DIR)/libmodule44.a

$(MODULE44_OBJ): $(MODULE44_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE44_LIB): $(MODULE44_OBJ)
	ar rcs $@ $<

module44: $(MODULE44_LIB)

module44-test: $(MODULE44_LIB)
	@echo "Testing module 44"
	@./test_module44

module44-clean:
	rm -f $(MODULE44_OBJ) $(MODULE44_LIB)

# Module 45
MODULE45_SRC = $(SRC_DIR)/module45.c
MODULE45_OBJ = $(BUILD_DIR)/module45.o
MODULE45_LIB = $(BUILD_DIR)/libmodule45.a

$(MODULE45_OBJ): $(MODULE45_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE45_LIB): $(MODULE45_OBJ)
	ar rcs $@ $<

module45: $(MODULE45_LIB)

module45-test: $(MODULE45_LIB)
	@echo "Testing module 45"
	@./test_module45

module45-clean:
	rm -f $(MODULE45_OBJ) $(MODULE45_LIB)

# Module 46
MODULE46_SRC = $(SRC_DIR)/module46.c
MODULE46_OBJ = $(BUILD_DIR)/module46.o
MODULE46_LIB = $(BUILD_DIR)/libmodule46.a

$(MODULE46_OBJ): $(MODULE46_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE46_LIB): $(MODULE46_OBJ)
	ar rcs $@ $<

module46: $(MODULE46_LIB)

module46-test: $(MODULE46_LIB)
	@echo "Testing module 46"
	@./test_module46

module46-clean:
	rm -f $(MODULE46_OBJ) $(MODULE46_LIB)

# Module 47
MODULE47_SRC = $(SRC_DIR)/module47.c
MODULE47_OBJ = $(BUILD_DIR)/module47.o
MODULE47_LIB = $(BUILD_DIR)/libmodule47.a

$(MODULE47_OBJ): $(MODULE47_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE47_LIB): $(MODULE47_OBJ)
	ar rcs $@ $<

module47: $(MODULE47_LIB)

module47-test: $(MODULE47_LIB)
	@echo "Testing module 47"
	@./test_module47

module47-clean:
	rm -f $(MODULE47_OBJ) $(MODULE47_LIB)

# Module 48
MODULE48_SRC = $(SRC_DIR)/module48.c
MODULE48_OBJ = $(BUILD_DIR)/module48.o
MODULE48_LIB = $(BUILD_DIR)/libmodule48.a

$(MODULE48_OBJ): $(MODULE48_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE48_LIB): $(MODULE48_OBJ)
	ar rcs $@ $<

module48: $(MODULE48_LIB)

module48-test: $(MODULE48_LIB)
	@echo "Testing module 48"
	@./test_module48

module48-clean:
	rm -f $(MODULE48_OBJ) $(MODULE48_LIB)

# Module 49
MODULE49_SRC = $(SRC_DIR)/module49.c
MODULE49_OBJ = $(BUILD_DIR)/module49.o
MODULE49_LIB = $(BUILD_DIR)/libmodule49.a

$(MODULE49_OBJ): $(MODULE49_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE49_LIB): $(MODULE49_OBJ)
	ar rcs $@ $<

module49: $(MODULE49_LIB)

module49-test: $(MODULE49_LIB)
	@echo "Testing module 49"
	@./test_module49

module49-clean:
	rm -f $(MODULE49_OBJ) $(MODULE49_LIB)

# Module 50
MODULE50_SRC = $(SRC_DIR)/module50.c
MODULE50_OBJ = $(BUILD_DIR)/module50.o
MODULE50_LIB = $(BUILD_DIR)/libmodule50.a

$(MODULE50_OBJ): $(MODULE50_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE50_LIB): $(MODULE50_OBJ)
	ar rcs $@ $<

module50: $(MODULE50_LIB)

module50-test: $(MODULE50_LIB)
	@echo "Testing module 50"
	@./test_module50

module50-clean:
	rm -f $(MODULE50_OBJ) $(MODULE50_LIB)

# Module 51
MODULE51_SRC = $(SRC_DIR)/module51.c
MODULE51_OBJ = $(BUILD_DIR)/module51.o
MODULE51_LIB = $(BUILD_DIR)/libmodule51.a

$(MODULE51_OBJ): $(MODULE51_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE51_LIB): $(MODULE51_OBJ)
	ar rcs $@ $<

module51: $(MODULE51_LIB)

module51-test: $(MODULE51_LIB)
	@echo "Testing module 51"
	@./test_module51

module51-clean:
	rm -f $(MODULE51_OBJ) $(MODULE51_LIB)

# Module 52
MODULE52_SRC = $(SRC_DIR)/module52.c
MODULE52_OBJ = $(BUILD_DIR)/module52.o
MODULE52_LIB = $(BUILD_DIR)/libmodule52.a

$(MODULE52_OBJ): $(MODULE52_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE52_LIB): $(MODULE52_OBJ)
	ar rcs $@ $<

module52: $(MODULE52_LIB)

module52-test: $(MODULE52_LIB)
	@echo "Testing module 52"
	@./test_module52

module52-clean:
	rm -f $(MODULE52_OBJ) $(MODULE52_LIB)

# Module 53
MODULE53_SRC = $(SRC_DIR)/module53.c
MODULE53_OBJ = $(BUILD_DIR)/module53.o
MODULE53_LIB = $(BUILD_DIR)/libmodule53.a

$(MODULE53_OBJ): $(MODULE53_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE53_LIB): $(MODULE53_OBJ)
	ar rcs $@ $<

module53: $(MODULE53_LIB)

module53-test: $(MODULE53_LIB)
	@echo "Testing module 53"
	@./test_module53

module53-clean:
	rm -f $(MODULE53_OBJ) $(MODULE53_LIB)

# Module 54
MODULE54_SRC = $(SRC_DIR)/module54.c
MODULE54_OBJ = $(BUILD_DIR)/module54.o
MODULE54_LIB = $(BUILD_DIR)/libmodule54.a

$(MODULE54_OBJ): $(MODULE54_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE54_LIB): $(MODULE54_OBJ)
	ar rcs $@ $<

module54: $(MODULE54_LIB)

module54-test: $(MODULE54_LIB)
	@echo "Testing module 54"
	@./test_module54

module54-clean:
	rm -f $(MODULE54_OBJ) $(MODULE54_LIB)

# Module 55
MODULE55_SRC = $(SRC_DIR)/module55.c
MODULE55_OBJ = $(BUILD_DIR)/module55.o
MODULE55_LIB = $(BUILD_DIR)/libmodule55.a

$(MODULE55_OBJ): $(MODULE55_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE55_LIB): $(MODULE55_OBJ)
	ar rcs $@ $<

module55: $(MODULE55_LIB)

module55-test: $(MODULE55_LIB)
	@echo "Testing module 55"
	@./test_module55

module55-clean:
	rm -f $(MODULE55_OBJ) $(MODULE55_LIB)

# Module 56
MODULE56_SRC = $(SRC_DIR)/module56.c
MODULE56_OBJ = $(BUILD_DIR)/module56.o
MODULE56_LIB = $(BUILD_DIR)/libmodule56.a

$(MODULE56_OBJ): $(MODULE56_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE56_LIB): $(MODULE56_OBJ)
	ar rcs $@ $<

module56: $(MODULE56_LIB)

module56-test: $(MODULE56_LIB)
	@echo "Testing module 56"
	@./test_module56

module56-clean:
	rm -f $(MODULE56_OBJ) $(MODULE56_LIB)

# Module 57
MODULE57_SRC = $(SRC_DIR)/module57.c
MODULE57_OBJ = $(BUILD_DIR)/module57.o
MODULE57_LIB = $(BUILD_DIR)/libmodule57.a

$(MODULE57_OBJ): $(MODULE57_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE57_LIB): $(MODULE57_OBJ)
	ar rcs $@ $<

module57: $(MODULE57_LIB)

module57-test: $(MODULE57_LIB)
	@echo "Testing module 57"
	@./test_module57

module57-clean:
	rm -f $(MODULE57_OBJ) $(MODULE57_LIB)

# Module 58
MODULE58_SRC = $(SRC_DIR)/module58.c
MODULE58_OBJ = $(BUILD_DIR)/module58.o
MODULE58_LIB = $(BUILD_DIR)/libmodule58.a

$(MODULE58_OBJ): $(MODULE58_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE58_LIB): $(MODULE58_OBJ)
	ar rcs $@ $<

module58: $(MODULE58_LIB)

module58-test: $(MODULE58_LIB)
	@echo "Testing module 58"
	@./test_module58

module58-clean:
	rm -f $(MODULE58_OBJ) $(MODULE58_LIB)

# Module 59
MODULE59_SRC = $(SRC_DIR)/module59.c
MODULE59_OBJ = $(BUILD_DIR)/module59.o
MODULE59_LIB = $(BUILD_DIR)/libmodule59.a

$(MODULE59_OBJ): $(MODULE59_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE59_LIB): $(MODULE59_OBJ)
	ar rcs $@ $<

module59: $(MODULE59_LIB)

module59-test: $(MODULE59_LIB)
	@echo "Testing module 59"
	@./test_module59

module59-clean:
	rm -f $(MODULE59_OBJ) $(MODULE59_LIB)

# Module 60
MODULE60_SRC = $(SRC_DIR)/module60.c
MODULE60_OBJ = $(BUILD_DIR)/module60.o
MODULE60_LIB = $(BUILD_DIR)/libmodule60.a

$(MODULE60_OBJ): $(MODULE60_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE60_LIB): $(MODULE60_OBJ)
	ar rcs $@ $<

module60: $(MODULE60_LIB)

module60-test: $(MODULE60_LIB)
	@echo "Testing module 60"
	@./test_module60

module60-clean:
	rm -f $(MODULE60_OBJ) $(MODULE60_LIB)

# Module 61
MODULE61_SRC = $(SRC_DIR)/module61.c
MODULE61_OBJ = $(BUILD_DIR)/module61.o
MODULE61_LIB = $(BUILD_DIR)/libmodule61.a

$(MODULE61_OBJ): $(MODULE61_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE61_LIB): $(MODULE61_OBJ)
	ar rcs $@ $<

module61: $(MODULE61_LIB)

module61-test: $(MODULE61_LIB)
	@echo "Testing module 61"
	@./test_module61

module61-clean:
	rm -f $(MODULE61_OBJ) $(MODULE61_LIB)

# Module 62
MODULE62_SRC = $(SRC_DIR)/module62.c
MODULE62_OBJ = $(BUILD_DIR)/module62.o
MODULE62_LIB = $(BUILD_DIR)/libmodule62.a

$(MODULE62_OBJ): $(MODULE62_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE62_LIB): $(MODULE62_OBJ)
	ar rcs $@ $<

module62: $(MODULE62_LIB)

module62-test: $(MODULE62_LIB)
	@echo "Testing module 62"
	@./test_module62

module62-clean:
	rm -f $(MODULE62_OBJ) $(MODULE62_LIB)

# Module 63
MODULE63_SRC = $(SRC_DIR)/module63.c
MODULE63_OBJ = $(BUILD_DIR)/module63.o
MODULE63_LIB = $(BUILD_DIR)/libmodule63.a

$(MODULE63_OBJ): $(MODULE63_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE63_LIB): $(MODULE63_OBJ)
	ar rcs $@ $<

module63: $(MODULE63_LIB)

module63-test: $(MODULE63_LIB)
	@echo "Testing module 63"
	@./test_module63

module63-clean:
	rm -f $(MODULE63_OBJ) $(MODULE63_LIB)

# Module 64
MODULE64_SRC = $(SRC_DIR)/module64.c
MODULE64_OBJ = $(BUILD_DIR)/module64.o
MODULE64_LIB = $(BUILD_DIR)/libmodule64.a

$(MODULE64_OBJ): $(MODULE64_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE64_LIB): $(MODULE64_OBJ)
	ar rcs $@ $<

module64: $(MODULE64_LIB)

module64-test: $(MODULE64_LIB)
	@echo "Testing module 64"
	@./test_module64

module64-clean:
	rm -f $(MODULE64_OBJ) $(MODULE64_LIB)

# Module 65
MODULE65_SRC = $(SRC_DIR)/module65.c
MODULE65_OBJ = $(BUILD_DIR)/module65.o
MODULE65_LIB = $(BUILD_DIR)/libmodule65.a

$(MODULE65_OBJ): $(MODULE65_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE65_LIB): $(MODULE65_OBJ)
	ar rcs $@ $<

module65: $(MODULE65_LIB)

module65-test: $(MODULE65_LIB)
	@echo "Testing module 65"
	@./test_module65

module65-clean:
	rm -f $(MODULE65_OBJ) $(MODULE65_LIB)

# Module 66
MODULE66_SRC = $(SRC_DIR)/module66.c
MODULE66_OBJ = $(BUILD_DIR)/module66.o
MODULE66_LIB = $(BUILD_DIR)/libmodule66.a

$(MODULE66_OBJ): $(MODULE66_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE66_LIB): $(MODULE66_OBJ)
	ar rcs $@ $<

module66: $(MODULE66_LIB)

module66-test: $(MODULE66_LIB)
	@echo "Testing module 66"
	@./test_module66

module66-clean:
	rm -f $(MODULE66_OBJ) $(MODULE66_LIB)

# Module 67
MODULE67_SRC = $(SRC_DIR)/module67.c
MODULE67_OBJ = $(BUILD_DIR)/module67.o
MODULE67_LIB = $(BUILD_DIR)/libmodule67.a

$(MODULE67_OBJ): $(MODULE67_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE67_LIB): $(MODULE67_OBJ)
	ar rcs $@ $<

module67: $(MODULE67_LIB)

module67-test: $(MODULE67_LIB)
	@echo "Testing module 67"
	@./test_module67

module67-clean:
	rm -f $(MODULE67_OBJ) $(MODULE67_LIB)

# Module 68
MODULE68_SRC = $(SRC_DIR)/module68.c
MODULE68_OBJ = $(BUILD_DIR)/module68.o
MODULE68_LIB = $(BUILD_DIR)/libmodule68.a

$(MODULE68_OBJ): $(MODULE68_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE68_LIB): $(MODULE68_OBJ)
	ar rcs $@ $<

module68: $(MODULE68_LIB)

module68-test: $(MODULE68_LIB)
	@echo "Testing module 68"
	@./test_module68

module68-clean:
	rm -f $(MODULE68_OBJ) $(MODULE68_LIB)

# Module 69
MODULE69_SRC = $(SRC_DIR)/module69.c
MODULE69_OBJ = $(BUILD_DIR)/module69.o
MODULE69_LIB = $(BUILD_DIR)/libmodule69.a

$(MODULE69_OBJ): $(MODULE69_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE69_LIB): $(MODULE69_OBJ)
	ar rcs $@ $<

module69: $(MODULE69_LIB)

module69-test: $(MODULE69_LIB)
	@echo "Testing module 69"
	@./test_module69

module69-clean:
	rm -f $(MODULE69_OBJ) $(MODULE69_LIB)

# Module 70
MODULE70_SRC = $(SRC_DIR)/module70.c
MODULE70_OBJ = $(BUILD_DIR)/module70.o
MODULE70_LIB = $(BUILD_DIR)/libmodule70.a

$(MODULE70_OBJ): $(MODULE70_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE70_LIB): $(MODULE70_OBJ)
	ar rcs $@ $<

module70: $(MODULE70_LIB)

module70-test: $(MODULE70_LIB)
	@echo "Testing module 70"
	@./test_module70

module70-clean:
	rm -f $(MODULE70_OBJ) $(MODULE70_LIB)

# Module 71
MODULE71_SRC = $(SRC_DIR)/module71.c
MODULE71_OBJ = $(BUILD_DIR)/module71.o
MODULE71_LIB = $(BUILD_DIR)/libmodule71.a

$(MODULE71_OBJ): $(MODULE71_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE71_LIB): $(MODULE71_OBJ)
	ar rcs $@ $<

module71: $(MODULE71_LIB)

module71-test: $(MODULE71_LIB)
	@echo "Testing module 71"
	@./test_module71

module71-clean:
	rm -f $(MODULE71_OBJ) $(MODULE71_LIB)

# Module 72
MODULE72_SRC = $(SRC_DIR)/module72.c
MODULE72_OBJ = $(BUILD_DIR)/module72.o
MODULE72_LIB = $(BUILD_DIR)/libmodule72.a

$(MODULE72_OBJ): $(MODULE72_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE72_LIB): $(MODULE72_OBJ)
	ar rcs $@ $<

module72: $(MODULE72_LIB)

module72-test: $(MODULE72_LIB)
	@echo "Testing module 72"
	@./test_module72

module72-clean:
	rm -f $(MODULE72_OBJ) $(MODULE72_LIB)

# Module 73
MODULE73_SRC = $(SRC_DIR)/module73.c
MODULE73_OBJ = $(BUILD_DIR)/module73.o
MODULE73_LIB = $(BUILD_DIR)/libmodule73.a

$(MODULE73_OBJ): $(MODULE73_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE73_LIB): $(MODULE73_OBJ)
	ar rcs $@ $<

module73: $(MODULE73_LIB)

module73-test: $(MODULE73_LIB)
	@echo "Testing module 73"
	@./test_module73

module73-clean:
	rm -f $(MODULE73_OBJ) $(MODULE73_LIB)

# Module 74
MODULE74_SRC = $(SRC_DIR)/module74.c
MODULE74_OBJ = $(BUILD_DIR)/module74.o
MODULE74_LIB = $(BUILD_DIR)/libmodule74.a

$(MODULE74_OBJ): $(MODULE74_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE74_LIB): $(MODULE74_OBJ)
	ar rcs $@ $<

module74: $(MODULE74_LIB)

module74-test: $(MODULE74_LIB)
	@echo "Testing module 74"
	@./test_module74

module74-clean:
	rm -f $(MODULE74_OBJ) $(MODULE74_LIB)

# Module 75
MODULE75_SRC = $(SRC_DIR)/module75.c
MODULE75_OBJ = $(BUILD_DIR)/module75.o
MODULE75_LIB = $(BUILD_DIR)/libmodule75.a

$(MODULE75_OBJ): $(MODULE75_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE75_LIB): $(MODULE75_OBJ)
	ar rcs $@ $<

module75: $(MODULE75_LIB)

module75-test: $(MODULE75_LIB)
	@echo "Testing module 75"
	@./test_module75

module75-clean:
	rm -f $(MODULE75_OBJ) $(MODULE75_LIB)

# Module 76
MODULE76_SRC = $(SRC_DIR)/module76.c
MODULE76_OBJ = $(BUILD_DIR)/module76.o
MODULE76_LIB = $(BUILD_DIR)/libmodule76.a

$(MODULE76_OBJ): $(MODULE76_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE76_LIB): $(MODULE76_OBJ)
	ar rcs $@ $<

module76: $(MODULE76_LIB)

module76-test: $(MODULE76_LIB)
	@echo "Testing module 76"
	@./test_module76

module76-clean:
	rm -f $(MODULE76_OBJ) $(MODULE76_LIB)

# Module 77
MODULE77_SRC = $(SRC_DIR)/module77.c
MODULE77_OBJ = $(BUILD_DIR)/module77.o
MODULE77_LIB = $(BUILD_DIR)/libmodule77.a

$(MODULE77_OBJ): $(MODULE77_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE77_LIB): $(MODULE77_OBJ)
	ar rcs $@ $<

module77: $(MODULE77_LIB)

module77-test: $(MODULE77_LIB)
	@echo "Testing module 77"
	@./test_module77

module77-clean:
	rm -f $(MODULE77_OBJ) $(MODULE77_LIB)

# Module 78
MODULE78_SRC = $(SRC_DIR)/module78.c
MODULE78_OBJ = $(BUILD_DIR)/module78.o
MODULE78_LIB = $(BUILD_DIR)/libmodule78.a

$(MODULE78_OBJ): $(MODULE78_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE78_LIB): $(MODULE78_OBJ)
	ar rcs $@ $<

module78: $(MODULE78_LIB)

module78-test: $(MODULE78_LIB)
	@echo "Testing module 78"
	@./test_module78

module78-clean:
	rm -f $(MODULE78_OBJ) $(MODULE78_LIB)

# Module 79
MODULE79_SRC = $(SRC_DIR)/module79.c
MODULE79_OBJ = $(BUILD_DIR)/module79.o
MODULE79_LIB = $(BUILD_DIR)/libmodule79.a

$(MODULE79_OBJ): $(MODULE79_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE79_LIB): $(MODULE79_OBJ)
	ar rcs $@ $<

module79: $(MODULE79_LIB)

module79-test: $(MODULE79_LIB)
	@echo "Testing module 79"
	@./test_module79

module79-clean:
	rm -f $(MODULE79_OBJ) $(MODULE79_LIB)

# Module 80
MODULE80_SRC = $(SRC_DIR)/module80.c
MODULE80_OBJ = $(BUILD_DIR)/module80.o
MODULE80_LIB = $(BUILD_DIR)/libmodule80.a

$(MODULE80_OBJ): $(MODULE80_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE80_LIB): $(MODULE80_OBJ)
	ar rcs $@ $<

module80: $(MODULE80_LIB)

module80-test: $(MODULE80_LIB)
	@echo "Testing module 80"
	@./test_module80

module80-clean:
	rm -f $(MODULE80_OBJ) $(MODULE80_LIB)

# Module 81
MODULE81_SRC = $(SRC_DIR)/module81.c
MODULE81_OBJ = $(BUILD_DIR)/module81.o
MODULE81_LIB = $(BUILD_DIR)/libmodule81.a

$(MODULE81_OBJ): $(MODULE81_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE81_LIB): $(MODULE81_OBJ)
	ar rcs $@ $<

module81: $(MODULE81_LIB)

module81-test: $(MODULE81_LIB)
	@echo "Testing module 81"
	@./test_module81

module81-clean:
	rm -f $(MODULE81_OBJ) $(MODULE81_LIB)

# Module 82
MODULE82_SRC = $(SRC_DIR)/module82.c
MODULE82_OBJ = $(BUILD_DIR)/module82.o
MODULE82_LIB = $(BUILD_DIR)/libmodule82.a

$(MODULE82_OBJ): $(MODULE82_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE82_LIB): $(MODULE82_OBJ)
	ar rcs $@ $<

module82: $(MODULE82_LIB)

module82-test: $(MODULE82_LIB)
	@echo "Testing module 82"
	@./test_module82

module82-clean:
	rm -f $(MODULE82_OBJ) $(MODULE82_LIB)

# Module 83
MODULE83_SRC = $(SRC_DIR)/module83.c
MODULE83_OBJ = $(BUILD_DIR)/module83.o
MODULE83_LIB = $(BUILD_DIR)/libmodule83.a

$(MODULE83_OBJ): $(MODULE83_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE83_LIB): $(MODULE83_OBJ)
	ar rcs $@ $<

module83: $(MODULE83_LIB)

module83-test: $(MODULE83_LIB)
	@echo "Testing module 83"
	@./test_module83

module83-clean:
	rm -f $(MODULE83_OBJ) $(MODULE83_LIB)

# Module 84
MODULE84_SRC = $(SRC_DIR)/module84.c
MODULE84_OBJ = $(BUILD_DIR)/module84.o
MODULE84_LIB = $(BUILD_DIR)/libmodule84.a

$(MODULE84_OBJ): $(MODULE84_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE84_LIB): $(MODULE84_OBJ)
	ar rcs $@ $<

module84: $(MODULE84_LIB)

module84-test: $(MODULE84_LIB)
	@echo "Testing module 84"
	@./test_module84

module84-clean:
	rm -f $(MODULE84_OBJ) $(MODULE84_LIB)

# Module 85
MODULE85_SRC = $(SRC_DIR)/module85.c
MODULE85_OBJ = $(BUILD_DIR)/module85.o
MODULE85_LIB = $(BUILD_DIR)/libmodule85.a

$(MODULE85_OBJ): $(MODULE85_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE85_LIB): $(MODULE85_OBJ)
	ar rcs $@ $<

module85: $(MODULE85_LIB)

module85-test: $(MODULE85_LIB)
	@echo "Testing module 85"
	@./test_module85

module85-clean:
	rm -f $(MODULE85_OBJ) $(MODULE85_LIB)

# Module 86
MODULE86_SRC = $(SRC_DIR)/module86.c
MODULE86_OBJ = $(BUILD_DIR)/module86.o
MODULE86_LIB = $(BUILD_DIR)/libmodule86.a

$(MODULE86_OBJ): $(MODULE86_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE86_LIB): $(MODULE86_OBJ)
	ar rcs $@ $<

module86: $(MODULE86_LIB)

module86-test: $(MODULE86_LIB)
	@echo "Testing module 86"
	@./test_module86

module86-clean:
	rm -f $(MODULE86_OBJ) $(MODULE86_LIB)

# Module 87
MODULE87_SRC = $(SRC_DIR)/module87.c
MODULE87_OBJ = $(BUILD_DIR)/module87.o
MODULE87_LIB = $(BUILD_DIR)/libmodule87.a

$(MODULE87_OBJ): $(MODULE87_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE87_LIB): $(MODULE87_OBJ)
	ar rcs $@ $<

module87: $(MODULE87_LIB)

module87-test: $(MODULE87_LIB)
	@echo "Testing module 87"
	@./test_module87

module87-clean:
	rm -f $(MODULE87_OBJ) $(MODULE87_LIB)

# Module 88
MODULE88_SRC = $(SRC_DIR)/module88.c
MODULE88_OBJ = $(BUILD_DIR)/module88.o
MODULE88_LIB = $(BUILD_DIR)/libmodule88.a

$(MODULE88_OBJ): $(MODULE88_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE88_LIB): $(MODULE88_OBJ)
	ar rcs $@ $<

module88: $(MODULE88_LIB)

module88-test: $(MODULE88_LIB)
	@echo "Testing module 88"
	@./test_module88

module88-clean:
	rm -f $(MODULE88_OBJ) $(MODULE88_LIB)

# Module 89
MODULE89_SRC = $(SRC_DIR)/module89.c
MODULE89_OBJ = $(BUILD_DIR)/module89.o
MODULE89_LIB = $(BUILD_DIR)/libmodule89.a

$(MODULE89_OBJ): $(MODULE89_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE89_LIB): $(MODULE89_OBJ)
	ar rcs $@ $<

module89: $(MODULE89_LIB)

module89-test: $(MODULE89_LIB)
	@echo "Testing module 89"
	@./test_module89

module89-clean:
	rm -f $(MODULE89_OBJ) $(MODULE89_LIB)

# Module 90
MODULE90_SRC = $(SRC_DIR)/module90.c
MODULE90_OBJ = $(BUILD_DIR)/module90.o
MODULE90_LIB = $(BUILD_DIR)/libmodule90.a

$(MODULE90_OBJ): $(MODULE90_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE90_LIB): $(MODULE90_OBJ)
	ar rcs $@ $<

module90: $(MODULE90_LIB)

module90-test: $(MODULE90_LIB)
	@echo "Testing module 90"
	@./test_module90

module90-clean:
	rm -f $(MODULE90_OBJ) $(MODULE90_LIB)

# Module 91
MODULE91_SRC = $(SRC_DIR)/module91.c
MODULE91_OBJ = $(BUILD_DIR)/module91.o
MODULE91_LIB = $(BUILD_DIR)/libmodule91.a

$(MODULE91_OBJ): $(MODULE91_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE91_LIB): $(MODULE91_OBJ)
	ar rcs $@ $<

module91: $(MODULE91_LIB)

module91-test: $(MODULE91_LIB)
	@echo "Testing module 91"
	@./test_module91

module91-clean:
	rm -f $(MODULE91_OBJ) $(MODULE91_LIB)

# Module 92
MODULE92_SRC = $(SRC_DIR)/module92.c
MODULE92_OBJ = $(BUILD_DIR)/module92.o
MODULE92_LIB = $(BUILD_DIR)/libmodule92.a

$(MODULE92_OBJ): $(MODULE92_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE92_LIB): $(MODULE92_OBJ)
	ar rcs $@ $<

module92: $(MODULE92_LIB)

module92-test: $(MODULE92_LIB)
	@echo "Testing module 92"
	@./test_module92

module92-clean:
	rm -f $(MODULE92_OBJ) $(MODULE92_LIB)

# Module 93
MODULE93_SRC = $(SRC_DIR)/module93.c
MODULE93_OBJ = $(BUILD_DIR)/module93.o
MODULE93_LIB = $(BUILD_DIR)/libmodule93.a

$(MODULE93_OBJ): $(MODULE93_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE93_LIB): $(MODULE93_OBJ)
	ar rcs $@ $<

module93: $(MODULE93_LIB)

module93-test: $(MODULE93_LIB)
	@echo "Testing module 93"
	@./test_module93

module93-clean:
	rm -f $(MODULE93_OBJ) $(MODULE93_LIB)

# Module 94
MODULE94_SRC = $(SRC_DIR)/module94.c
MODULE94_OBJ = $(BUILD_DIR)/module94.o
MODULE94_LIB = $(BUILD_DIR)/libmodule94.a

$(MODULE94_OBJ): $(MODULE94_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE94_LIB): $(MODULE94_OBJ)
	ar rcs $@ $<

module94: $(MODULE94_LIB)

module94-test: $(MODULE94_LIB)
	@echo "Testing module 94"
	@./test_module94

module94-clean:
	rm -f $(MODULE94_OBJ) $(MODULE94_LIB)

# Module 95
MODULE95_SRC = $(SRC_DIR)/module95.c
MODULE95_OBJ = $(BUILD_DIR)/module95.o
MODULE95_LIB = $(BUILD_DIR)/libmodule95.a

$(MODULE95_OBJ): $(MODULE95_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE95_LIB): $(MODULE95_OBJ)
	ar rcs $@ $<

module95: $(MODULE95_LIB)

module95-test: $(MODULE95_LIB)
	@echo "Testing module 95"
	@./test_module95

module95-clean:
	rm -f $(MODULE95_OBJ) $(MODULE95_LIB)

# Module 96
MODULE96_SRC = $(SRC_DIR)/module96.c
MODULE96_OBJ = $(BUILD_DIR)/module96.o
MODULE96_LIB = $(BUILD_DIR)/libmodule96.a

$(MODULE96_OBJ): $(MODULE96_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE96_LIB): $(MODULE96_OBJ)
	ar rcs $@ $<

module96: $(MODULE96_LIB)

module96-test: $(MODULE96_LIB)
	@echo "Testing module 96"
	@./test_module96

module96-clean:
	rm -f $(MODULE96_OBJ) $(MODULE96_LIB)

# Module 97
MODULE97_SRC = $(SRC_DIR)/module97.c
MODULE97_OBJ = $(BUILD_DIR)/module97.o
MODULE97_LIB = $(BUILD_DIR)/libmodule97.a

$(MODULE97_OBJ): $(MODULE97_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE97_LIB): $(MODULE97_OBJ)
	ar rcs $@ $<

module97: $(MODULE97_LIB)

module97-test: $(MODULE97_LIB)
	@echo "Testing module 97"
	@./test_module97

module97-clean:
	rm -f $(MODULE97_OBJ) $(MODULE97_LIB)

# Module 98
MODULE98_SRC = $(SRC_DIR)/module98.c
MODULE98_OBJ = $(BUILD_DIR)/module98.o
MODULE98_LIB = $(BUILD_DIR)/libmodule98.a

$(MODULE98_OBJ): $(MODULE98_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE98_LIB): $(MODULE98_OBJ)
	ar rcs $@ $<

module98: $(MODULE98_LIB)

module98-test: $(MODULE98_LIB)
	@echo "Testing module 98"
	@./test_module98

module98-clean:
	rm -f $(MODULE98_OBJ) $(MODULE98_LIB)

# Module 99
MODULE99_SRC = $(SRC_DIR)/module99.c
MODULE99_OBJ = $(BUILD_DIR)/module99.o
MODULE99_LIB = $(BUILD_DIR)/libmodule99.a

$(MODULE99_OBJ): $(MODULE99_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE99_LIB): $(MODULE99_OBJ)
	ar rcs $@ $<

module99: $(MODULE99_LIB)

module99-test: $(MODULE99_LIB)
	@echo "Testing module 99"
	@./test_module99

module99-clean:
	rm -f $(MODULE99_OBJ) $(MODULE99_LIB)

# Module 100
MODULE100_SRC = $(SRC_DIR)/module100.c
MODULE100_OBJ = $(BUILD_DIR)/module100.o
MODULE100_LIB = $(BUILD_DIR)/libmodule100.a

$(MODULE100_OBJ): $(MODULE100_SRC)
	$(CC) $(CFLAGS) -c $< -o $@

$(MODULE100_LIB): $(MODULE100_OBJ)
	ar rcs $@ $<

module100: $(MODULE100_LIB)

module100-test: $(MODULE100_LIB)
	@echo "Testing module 100"
	@./test_module100

module100-clean:
	rm -f $(MODULE100_OBJ) $(MODULE100_LIB)

all: module1 module2 module3 module4 module5 module6 module7 module8 module9 module10 module11 module12 module13 module14 module15 module16 module17 module18 module19 module20 module21 module22 module23 module24 module25 module26 module27 module28 module29 module30 module31 module32 module33 module34 module35 module36 module37 module38 module39 module40 module41 module42 module43 module44 module45 module46 module47 module48 module49 module50 module51 module52 module53 module54 module55 module56 module57 module58 module59 module60 module61 module62 module63 module64 module65 module66 module67 module68 module69 module70 module71 module72 module73 module74 module75 module76 module77 module78 module79 module80 module81 module82 module83 module84 module85 module86 module87 module88 module89 module90 module91 module92 module93 module94 module95 module96 module97 module98 module99 module100

clean: module1-clean module2-clean module3-clean module4-clean module5-clean module6-clean module7-clean module8-clean module9-clean module10-clean module11-clean module12-clean module13-clean module14-clean module15-clean module16-clean module17-clean module18-clean module19-clean module20-clean module21-clean module22-clean module23-clean module24-clean module25-clean module26-clean module27-clean module28-clean module29-clean module30-clean module31-clean module32-clean module33-clean module34-clean module35-clean module36-clean module37-clean module38-clean module39-clean module40-clean module41-clean module42-clean module43-clean module44-clean module45-clean module46-clean module47-clean module48-clean module49-clean module50-clean module51-clean module52-clean module53-clean module54-clean module55-clean module56-clean module57-clean module58-clean module59-clean module60-clean module61-clean module62-clean module63-clean module64-clean module65-clean module66-clean module67-clean module68-clean module69-clean module70-clean module71-clean module72-clean module73-clean module74-clean module75-clean module76-clean module77-clean module78-clean module79-clean module80-clean module81-clean module82-clean module83-clean module84-clean module85-clean module86-clean module87-clean module88-clean module89-clean module90-clean module91-clean module92-clean module93-clean module94-clean module95-clean module96-clean module97-clean module98-clean module99-clean module100-clean

test: module1-test module2-test module3-test module4-test module5-test module6-test module7-test module8-test module9-test module10-test module11-test module12-test module13-test module14-test module15-test module16-test module17-test module18-test module19-test module20-test module21-test module22-test module23-test module24-test module25-test module26-test module27-test module28-test module29-test module30-test module31-test module32-test module33-test module34-test module35-test module36-test module37-test module38-test module39-test module40-test module41-test module42-test module43-test module44-test module45-test module46-test module47-test module48-test module49-test module50-test module51-test module52-test module53-test module54-test module55-test module56-test module57-test module58-test module59-test module60-test module61-test module62-test module63-test module64-test module65-test module66-test module67-test module68-test module69-test module70-test module71-test module72-test module73-test module74-test module75-test module76-test module77-test module78-test module79-test module80-test module81-test module82-test module83-test module84-test module85-test module86-test module87-test module88-test module89-test module90-test module91-test module92-test module93-test module94-test module95-test module96-test module97-test module98-test module99-test module100-test

.SUFFIXES:
.DELETE_ON_ERROR:
