TEST_ARGS=""
if [ -n "$1" ]; then
  TEST_ARGS="-g $1"
fi

yarn run ts-mocha -p ./tsconfig.json --bail -t 1000000 tests/**/*.ts $TEST_ARGS
