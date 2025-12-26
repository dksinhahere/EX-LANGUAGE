
visible my_block(
    counter = 0,
    appName = "MyApp",
    maxUsers = 100
)

visible do_also(
    inc = 0
)

label visit[my_block, do_also] myFunction2() {
    counter = counter + 1
    inc = inc + 10
    kprint counter
}

label visit[my_block, do_also] myFunction() {
    counter = counter + 1
    kprint counter
    kprint inc
}

myFunction()
myFunction()
myFunction()
myFunction()

myFunction2()
myFunction()
