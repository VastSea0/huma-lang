fibonacci fonksiyon olsun n alsın {
    n <= 1 ise { n'i döndür }
    fibonacci(n - 1) + fibonacci(n - 2)'yi döndür
}

"--- Fibonacci Serisi ---"'yi yazdır
i = 0 olsun
i < 10 olduğu sürece {
    "fib(" + i + ") = " + fibonacci(i)'yi yazdır
    i = i + 1 olsun
}
