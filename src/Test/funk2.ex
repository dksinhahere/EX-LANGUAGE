
visible block (
    msg = "Hello"
)

label visit[block] greeting(name=uname, age=uage) {
    
    kprint (uname+" "+uage)
    kprint (msg)
}

greeting(name="Danishk", age="22")