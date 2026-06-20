================================================================
n0ne — Less syntax. More meaning.
================================================================

n0ne is a compiled, statically-typed, indentation-sensitive language
built on top of LLVM.

----------------------------------------------------------------
INSTALLATION
----------------------------------------------------------------

Linux / macOS:
  curl -fsSL https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.sh | sh

Windows (PowerShell):
  irm https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.ps1 | iex

The installer automatically sets up clang and adds n0ne to your PATH.
No manual compiler installation required.

----------------------------------------------------------------
GETTING STARTED
----------------------------------------------------------------

1. Create a file named hello.n0:

   task main
       print("hello world")

2. Run it:
   n0ne run hello.n0

3. Build a native executable:
   n0ne build hello.n0

More examples are in the examples/ directory.

----------------------------------------------------------------
LICENSE
----------------------------------------------------------------

MIT License. See LICENSE file for details.
