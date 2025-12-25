

#_define_ is_connected(signal)
[
    if(signal) [
        log(src="Yes, Connection Build")
    ] else [
        log(src="Failed, to connect")
    ]
]

#_define_ DEBUG [
    return true
]

#ifndef DEBUG
[
    #_define_ DEBUG
    [
        return true
    ]
]

label entry_main(define cla) &[visibility()]
[
    signal = true
    is_connected(true)
    

    #ifdef DEBUG 
    [
        log(src="Debug mode is On")
    ]

    #undef
    [
        DEBUG,
        is_connected
    ]
]




/*
    if(DEBUG)
    [
        log(src="Debug mode is On")
    ] else [
        log(src="Debug Mode is Off")
    ]
*/
