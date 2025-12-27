
_ifndef_ DEBUG
    
    _define_ DEBUG(x) [
        kprint name
        kprint "DEBUG IS ON"
        kprint msg
        kprint x
    ]

_endif_

_define_ ADDITION() [
    kprint x+y
]

label visit[] greeting() {
    
    name = "Danishk"
    msg = "Hello, budddy"

    #DEBUG(msg)
}

greeting()

x = 10
y = 20
#ADDITION()

_undef_ DEBUG
_undef_ ADDITION