# n0ne

================================================================
n0ne — Less syntax. More meaning.
================================================================

n0ne is a compiled, statically-typed language.
Target is native binaries via LLVM IR.


----------------------------------------------------------------
PACKAGES (CARGO WORKSPACE)
----------------------------------------------------------------

compiler/lexer         Hand-written tokenizer
compiler/parser        Recursive descent parser
compiler/ast           AST definitions
compiler/sema          Type checker and symbol table
compiler/codegen/llvm  LLVM IR emitter and runtime linker
compiler/cli           Command line interface
tools/formatter        Source code formatter


----------------------------------------------------------------
CI/CD PIPELINES
----------------------------------------------------------------

Continuous Integration:
  - Test matrix runs on Ubuntu, macOS, and Windows.

Continuous Delivery:
  - Automates release builds on version tag push (v*).
  - Packages pre-compiled binaries into tar.gz and zip files.
  - Automatically published to GitHub Releases.
