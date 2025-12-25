
/*
enum Battery [
    low,
    mid=2,
    high
]

switch (Battery.low) [
    case 0: [
        log(src="battery is low")
    ]
    default: [
        log(src="Battery is corrupted")
    ]
]

*/

label entry_point(define cla) &[visibility()] [ 
    
    enum Battery [
        low,
        mid,
        high
    ]

    enum SelOs [
        _linux_=1,
        _windows_=2,
        _macos_=3
    ]

    enum Mobile
    [
        Battery,
        SelOs
    ]

    log(src=Mobile.Battery.high)
    log(src=Mobile.SelOs._linux_)
]
