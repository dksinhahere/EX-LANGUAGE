


label greeting(name=uname, age=uage) {
    log uname + " How are You, " + uage

    label some(src=data) {
        log data
    }

    some(src="Hello, Suar")
}

greeting(name="Danishk", age="23")


count = 1

label @do {
    if count == 10 {
        jump done
    } else {
        jump do
    }
}

label @done {
    pass
}