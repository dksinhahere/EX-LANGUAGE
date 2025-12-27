
visible some {
    money = 55
    fabchar = 'C'
}

label visit[some], greeting(name=uname, age=uage) {
    kprint uname + " How are You, " + uage

    label some(src=data) {
        kprint data
    }

    some(src="Hello, Suar")

    kprint(some->money)
}

greeting(name="Danishk", age="23")


count = 1

label @done {
    kprint "Done!"
}

label @_do_ {
    kprint count
    count = count + 1
    if count == 10 {
        jump done
    } else {
        jump _do_
    }
}


if count <= 10
{
    jump _do_
} else {
    pass
}