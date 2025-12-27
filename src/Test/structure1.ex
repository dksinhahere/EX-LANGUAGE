struct student {
    constructor(self, name, age) {
        self.name = name
        self.age = age
        self.school = "My School"
    }
    
    print_details(self) {
        kprint self.name
        kprint self.age
    }

    get_details(self) {
        kprint name
    }
}

anoop = student::new("Anoop", 23)
anoop.print_details()

anoop.name = "Nikhil"
kprint anoop.name

anoop.get_details()