cd lib && make clean && pipx run compiledb make && cd .. &&
  cd kernel && make clean && pipx run compiledb make && cd ..
