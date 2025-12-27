
// implement structure in langauge
// First Example

struct student 
{
    constructor(self, name, age) {

        self.name = name
        self.age = age
        self.school = "My Wow School"
        self.pocket_money

    }

    // getter
    print_details(self) {
        kprint self.name
        kprint self.age
        kprint self.school
        kprint self.pocket_money
    }

    // Explicit setting value
    set(self, name, age, school, pocket_money) {
        self.name = name
        self.age = age
        self.school = school
        self.pocket_money = pocket_money
    }
}

anoop = student::new("anoop", 23, "achivement")
anoop.print_details()
anoop.set("anoop", 23, "achivement", 44.44)



// second example
struct book
{
    constructor(self, name, author, pages) {
        self.name = name
        self.author = author
        self.pages = pages
    }
}

letUcC = book::new("LetUsC", "Yashwant Kanetkar", 550)
printk letUcC.name
printk letUcC.author
printk letUcC.pages

letUcC.name = "Danishk"
printk letUcC.name