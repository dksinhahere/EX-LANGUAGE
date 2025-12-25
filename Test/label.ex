
label print_msg(define msg) &[visibility()]
[
    log(src=msg)
    unlabel nil
]

label main_entry(define cla) &[visibility()] [
    var1 = 0
    
    jump loop
    
    label @loop [
        if (var1 > 5) [
            jump end
        ]
        
        log(src=var1)
        var1=var1+1
        jump loop
    ]
    
    label @end [
        print_msg(msg="Hello The end")
    ]
]