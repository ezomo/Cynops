#!/bin/bash

input=$1
expectation=$2


assert() {
  expected="$1"
  actual="$2"
  if [ "$actual" = "$expected" ]; then
    echo "Success: Input $input => Output $actual"
  else
    echo "Failed: Input $input => Expected $expected, but got $actual"
    exit 1
  fi
}

# Store the command result in a variable
result=$(./run.sh "$input")

# Compare result with expectation
assert "$expectation" "$result"