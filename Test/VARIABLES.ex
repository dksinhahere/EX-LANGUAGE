
label entry_main(define cla) &[visibility()]
[

    /*
    //    DEFAULT VARIABLE

    _def_ (name, "danishk") = 10
    log(src=name*def)
    log(src=name*gen)

    */

    /*
        //    TTv Variable
    

    _ttv_ some = 11
    some = 12
    some = 13

    log(src=some)
    log(src=some*1)
    log(src=some*2)
    */


    /*
        //   DeadLock Variable
    

    _delock_ xyz = 45
    xyz*lock

    xyz = 55 // Throw Variable

    xyz*unlock
    xyz = 55 // Run

    log(src=xyz)
    
    xyz*kill
    xyz*is_alive // false

    xyz*revive
    xyz*is_alive // true

    */


    // Global Variable
    rooted var x = 54

    // Constant Variable
    eternal var y = 3.14

    // Define Empty
    define z

    // Create Variable Simple Variable
    name = "danishk"
    age = 44
    pow = math_pow(tar=2, arise=10)
    
]