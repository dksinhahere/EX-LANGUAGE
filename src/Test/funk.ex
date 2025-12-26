


label greeting(name=uname, age=uage) {
    print uname + " How are You, " + uage

    label some(src=data) {
        print data
    }

    some(src="Hello, Suar")
}

greeting(name="Danishk", age="23")


count = 1

label @done {
    print "Done!"
}

label @do {
    print count
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