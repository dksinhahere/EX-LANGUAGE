
// in my language, al least one constructor is very necessary
// new one can access constructor data directly
// No Structure Nesting because of (new one can access constructor data directly)
// but structure inheritance allowed

struct Car {
    constructor run(self, define name, define color, define engine, define year)
    [
        define name
        define color
        define engine
        rooted x = 10
        rooted year
        eternal tier=4
    ]
    
    public read() &[visibility()]
    [
        log(flatten_array=[self->name, self->color, self->engine, self->year])
    ]

    public write(define name, define color, define engine, define year) &[visibility()]
    [
        self->name = name
        self->color = color
        self->engine = engine
        self->year = year
    ]
}



label entry_main(define cla) &[visibility()]
[
    farari = new::Car(name="farari", color="blue", engine="4stoke", year=2025)
    farari -> read()
    farari -> write(name ="Toyota", color="green", engine="8stoke", year=2045)
    farari -> read()

    unlabel 0
]