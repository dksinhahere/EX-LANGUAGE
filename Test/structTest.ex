

struct Book
{
    constructor book(self, define name, define author, define release) [
        self->name = name
        self->author = author
        self->release = release
    ]
}


label entry_main(define cla) &[visibility()]
[
    book = new::Book(name="Let Us C", author="danishk", year=2019)

    log(src=book->name)
    log(src=book->author)
    log(src=book->release)
]
