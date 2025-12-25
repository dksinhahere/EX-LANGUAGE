#_define_ is_connected(signal)
[
    return true
]

label main_entry(define cla) &[visibility()] [
    if(is_connected(true)) [
        log(src="OK")
    ]
]