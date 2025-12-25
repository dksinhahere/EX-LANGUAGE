 
*[visible_soft (primary)]
[
    name = "danishk"
    age = 12
    roll = 55
]

*[visible_soft (secondary)]

[
    dog_name = "danishk"
    dog_age = 12
    dog_roll = 55
]

*[visible_hard (some)]
[
    dog_name = "danishk"
    dog_age = 12
    dog_roll = 55
]

label entry_main(define cla) &[visibility(primary, second, some)]
[
    log(src="Hello")
]