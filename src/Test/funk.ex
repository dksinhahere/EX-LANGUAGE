


label greeting(name=uname, age=uage) {
    log uname + " How are You, " + uage

    label some(src=data) {
        log data
    }

    some(src="Hello, Suar")
}

greeting(name="Danishk", age="23")


count = 1

label @done {
    log "Done!"
}

label @do {
    log count
    count = count + 1
    if count == 10 {
        jump done
    } else {
        jump do
    }
}


if count <= 10
{
    jump do
} else {
    pass
}