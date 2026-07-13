function add(a, b) {
  return a + b;
}

function fib(n) {
  if (n <= 2) {
    return n;
  }
  return fib(n - 1) + fib(n - 2);
}

console.log(add(1, 2));
console.log(fib(5));
