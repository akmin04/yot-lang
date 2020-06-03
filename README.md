# Yotc
LLVM frontend for yot - a toy language.

*Loosely based off of LLVM Kaleidoscope*

# Running
* Install LLVM 9.0
* Install yotc with `cargo install yotc`
* For automatic linking (a.k.a. default output format "executable"), `gcc` needs to be in PATH
* Usage: `yotc (path to file)`
* Run `yotc --help` for more options

# Yot Syntax
* Note: every variable is a 32-bit `int` and functions must return an `int` as well. Comparison operators return 1 or 0
* Functions:
    * Regular syntax 
        ```
        @sum[a, b] {
            -> a + b // -> is the return keyword
        }
        ```
    * Short-hand notation for just return statement
        ```
        @sum[a, b] -> a + b;
        ```
    * External functions
        ```
        @!print[_, _];
        ```
    * Calling a function
        ```
        sum(a, b);
        ```
* Variables:
    * Declaration with value
        ```
        @a = 5;
        ```
    * Declaration without value (will be assigned to trash value)
        ```
        @a;
        ```
    * Referencing a variable
        ```
        @b = a + 5;
        ```
* Operations
    * Available operations `=`, `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`.
        ```
        @a = (-b + 5) - 10 / -(5 - -2);
        ```
* Comments
    * Comments start with `//` and tokens are ignored until the end of the line
* Programs
    * A program consists of just top-level functions (no global variables yet)
    * `main` function entry point
* Example
    * See `examples/`
    * Run by first generating the object file of `equals_ten.yot` with `yotc equals_ten.yot -f object-file`
    * Compile and link `io.cpp` with `g++ io.cc equals_ten.o` to generate an executable

# Todo
* If, for, while statements
* LLVM IR optimization
* Support printing string literals
* Better compiler errors
    * Current errors are either vague or plain wrong and dont have any information about line number
    * Most errors are caught by LLVM, meaning more ugly and vague error messages
* More data types (float, bool, char)
* Testing
