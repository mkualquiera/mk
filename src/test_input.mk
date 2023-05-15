

my_file : my_file.c another_file.c
	gcc -o my_file my_file.c
	magic my_file

$clean :
	rm -f my_file

$all: my_file	

my_file :my_file.c another_file.c
    gcc -o my_file my_file.c
    magic my_file