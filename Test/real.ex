
label some(define msg) &[visibility()]
[
    log(src=msg)
]

label entry_main(define cla) &[visibility()]
[
    
    count=0

    jump repeat

    label @repeat
    [
        if(count >= 10) [
            jump end
        ] else [
            log(src=count)
            count = count+1
            jump repeat
        ]
    ]

    label @end [
        some(msg="======== ALL DONE ===========")
    ]

    unlabel 0
]